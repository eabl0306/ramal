# ADR 0019: Sincronizacion incremental: identidad por remoto normalizado, feed con cursor por ambito y concurrencia optimista

- Estado: aceptado
- Fecha: 2026-10-23

## Contexto

La sincronizacion personal (F2) tiene que dejar que varios dispositivos de la misma persona
vean los mismos workspaces, repositorios, cuentas vinculadas y preferencias, sin bajarse todo
en cada arranque y sin que dos equipos que editan a la vez se pisen.

Tres decisiones de esto son **contratos de datos**: cambiarlas despues no es refactorizar, es
migrar lo que ya hay en la base y en cada dispositivo.

1. **Como se identifica un repositorio** entre dispositivos. Si dos versiones del cliente
   deciden distinto, el mismo repositorio se parte en dos filas y el usuario ve duplicados.
2. **Como se piden los cambios desde la ultima vez.** De aqui depende que un borrado no se
   pierda, que es el fallo que no se nota hasta que alguien ve en un portatil un workspace que
   borro hace un mes.
3. **Como se resuelve que dos dispositivos escriban el mismo documento de ajustes.**

Hasta ahora las tres vivian repartidas entre [api-surface.md](../api-surface.md) y
[data-model.md](../data-model.md), descritas como endpoints y como tablas. Eso basta para
implementar, pero no dice **por que** son asi ni que alternativas se descartaron, y es
justo lo que hace falta el dia que alguien proponga "usar una marca de tiempo, que es mas
simple".

La mecanica del feed viene de Gastonomo, con dos cosas que Git Rudder necesita y Gastonomo no:
**borrados de verdad** y **datos compartidos entre varios usuarios** (los workspaces de una
organizacion).

## Decision

### La identidad de un repositorio es su remoto normalizado, y la calcula el servidor

La API **no guarda rutas locales**, ni las del propio dispositivo: una ruta como
`/home/marta/clientes/proyecto-secreto` dice cosas del usuario que no hacen falta y en otra
maquina no vale para nada. El registro local con las rutas vive en cada dispositivo.

La identidad de un repositorio con remoto es su **`remoteKey`**: la URL del remoto principal
normalizada. El remoto principal es `origin` si existe; si no, el unico que haya; si hay varios
sin `origin`, el primero por nombre.

Las reglas de la normalizacion: host en minusculas, sin esquema ni usuario, sin puerto si es el
de por defecto del esquema, sin `.git` ni barra final, y la forma `usuario@host:ruta` pasada a
`host/ruta`. La ruta se pasa a minusculas **solo en los hosts que no distinguen mayusculas**
(github.com, gitlab.com, bitbucket.org, dev.azure.com); en un Gitea propio `Equipo/API` y
`equipo/api` pueden ser dos repositorios distintos. La tabla de ejemplos esta en
[api-surface.md](../api-surface.md#identidad-el-remoto-normalizado-nunca-la-ruta).

Dos reglas que lo convierten en un contrato y no en una funcion de utilidad:

- **La normalizacion vive en un solo sitio**, `git-rudder-core-domain`, y es la misma en el motor
  y en la API.
- **La API nunca acepta un `remoteKey` del cliente.** Lo recalcula siempre a partir de
  `remoteUrl`. Un cliente viejo que normalizara distinto no puede partir un repositorio en dos
  si no es el quien decide la clave.

Cambiar una regla de normalizacion en el futuro es **una migracion de datos**: recalcular la
clave de cada fila y unir las que colisionen. Por eso la regla se versiona junto al esquema y no
se toca a la ligera.

Un repositorio **sin remoto** no tiene identidad global: su identidad es la pareja (dispositivo
de origen, id local) que genera el dispositivo, y en la web aparece como "Local en
{dispositivo}". No se intenta adivinar que dos carpetas sin remoto en dos equipos son "el mismo"
repositorio, porque no hay forma de saberlo.

### El cursor es un contador por ambito, no una marca de tiempo

`GET /v1/sync/changes?since=<cursor>` entrega lo que cambio. Hay **un contador por ambito**: uno
para el ambito personal del usuario y uno por cada organizacion. Cada cambio toma el siguiente
numero de su ambito con un `UPDATE ... RETURNING` con cerrojo de fila, y de ahi sale la garantia
que hace que el feed sea correcto: **cuando el numero N de un ambito es visible, ninguno menor de
ese ambito esta todavia por confirmar**.

Por que no las dos alternativas obvias:

- **Una secuencia global** no da esa garantia: dos transacciones pueden tomar numeros en un orden
  y confirmar en el inverso, asi que un lector puede ver el N y perderse el N-1 para siempre.
- **Una marca de tiempo** se salta filas del mismo milisegundo, y si el reloj del servidor va
  hacia atras se salta un rato entero.

Un contador **por ambito** y no uno por usuario porque un workspace compartido cambia para todos
los miembros a la vez: con un contador por usuario habria que escribir el cambio una vez por
miembro, y con uno por organizacion se escribe una vez y lo leen todos.

El cursor es **opaco** para el cliente: base64url de `{ "v": 1, "u": 42, "o": { "<orgId>": 17 } }`
firmado con HMAC. Es opaco a proposito, para que nadie construya uno a mano ni dependa de su
forma; la version `v` deja cambiarla sin romper a quien tenga uno viejo. Un cursor que no sea de
esta API es **422** `invalid_cursor`.

Cada ambito se corta en un tramo cerrado (se piden `limit + 1` cambios y el cursor avanza hasta
donde el lote esta completo), y `hasMore` avisa si alguno se quedo a medias. **El cliente avanza
su cursor despues de guardar el lote**, en la misma transaccion local que los datos, nunca al
recibirlo: si el proceso muere entre una cosa y la otra, repite el lote, que es inofensivo, en
vez de perderlo.

Sin `since` es el arranque en frio. Como el registro guarda una fila por entidad (la del ultimo
cambio) y no la historia, leer desde cero es leer el estado actual.

### Un borrado deja lapida, y un cursor demasiado viejo es 410, nunca un arranque en frio en silencio

Cada borrado deja una **lapida** con su numero de cambio, y las lapidas se purgan a los **90
dias**. Un cliente cuyo cursor en algun ambito sea anterior a la ultima lapida purgada recibe
**410** `cursor_expired`.

Esto es lo mas importante de este ADR, porque es la unica parte donde la respuesta facil esta
mal. Un 410 significa "se ha podido perder un borrado", y la unica respuesta correcta es una
**resincronizacion completa**: pedir sin `since` y, al terminar, borrar en local lo que no haya
llegado. Tratar el cursor caducado como "pues leo desde cero" **sin ese barrido final** dejaria
en el dispositivo cosas que ya no existen, para siempre y sin que nada lo avise: el feed solo
cuenta lo que cambio, y de lo borrado hace mas de 90 dias ya no queda ni rastro que contar.

El feed ademas **ensena el estado tal como lo ve quien pregunta**. Un workspace privado de otro
miembro vive en el ambito de la organizacion, pero a los demas les llega como `deleted`, no como
dato; si pasa de compartido a privado reciben el borrado, y si pasa de privado a compartido lo
reciben como nuevo. Recibir el `deleted` de algo que nunca se tuvo es el precio, y es inofensivo:
borrar lo que no se tiene no es un error.

### Las categorias en pausa las cumplen los dispositivos, no la API

Los ajustes de `GET /v1/me/sync-settings` dicen que categorias se sincronizan. Apagar una no
significa lo mismo en todas, y es a proposito:

- **Las dos que llevan secretos** (`linkedAccounts` y `aiKeys`) **borran sus datos de la nube** en
  la misma peticion, metadatos y blobs. Quien apaga "Cuentas vinculadas" espera que dejen de estar
  en la nube, no que se queden quietas ahi. Los dispositivos conservan sus copias locales y
  reciben los borrados como lapidas. Mientras sigan apagadas, la API rechaza cualquier escritura
  de esa categoria con **409** `sync_category_disabled`, para que un cliente viejo que no sepa de
  la pausa no vuelva a subir lo que el usuario quito.
- **Las otras cuatro se pausan**: los datos se quedan, los dispositivos dejan de subir y de
  aplicar cambios, y al encenderla otra vez se retoma. Apagar la apariencia porque se quiere un
  tema distinto en cada equipo no tiene por que borrar nada.

**El feed no filtra por categoria**: entrega todo lo que cambio y cada dispositivo ignora lo de
las categorias en pausa, lo guarda sin aplicarlo y, al reanudar, aplica la copia de la nube que ya
tiene. Filtrar en el servidor obligaria a que el cursor supiera con que ajustes se leyo cada
tramo: al reanudar una categoria habria que rebobinar su cursor, y no hay un cursor por categoria.
Entregar todo y decidir en el cliente deja el cursor con un solo significado.

De ahi sale una consecuencia que conviene tener escrita: **la pausa no la cumple el panel web**.
El panel es la copia de la nube, y pausar solo decide si los equipos la siguen; por eso
`PATCH /v1/me/preferences/{namespace}` sigue funcionando con la categoria apagada.

### Los documentos de ajustes se escriben con version, `If-Match` y merge patch

Los documentos que se sincronizan entre dispositivos (preferencias de Git, de apariencia y de IA,
los ajustes de sincronizacion y la cabecera de la boveda) llevan un `version` entero que la
respuesta devuelve tambien como `ETag`.

- Toda escritura exige `If-Match`. Sin la cabecera, **428** `precondition-required`.
- Si la version no coincide, **412** `precondition-failed` con `code: version_conflict` y el
  documento vigente en la extension `current`, para que el cliente reintente sin hacer un `GET`
  antes.
- Las preferencias se escriben con **JSON Merge Patch** (RFC 7396): el cliente manda solo los
  campos que el usuario cambio, y reintentar tras un 412 es volver a aplicar el mismo parche
  sobre `current`.

El merge patch no es un capricho: un cliente viejo que hiciera `PUT` del documento entero
borraria los campos que anadio una version mas nueva y que el no conoce. Con un parche, cada
cliente solo toca lo suyo. Por eso tambien **el servidor valida cada documento contra su esquema**,
estricto y sin campos desconocidos: tres clientes en versiones distintas leen el mismo documento,
y si uno guardara basura los otros dos se romperian.

No se usan marcas de tiempo por campo (la persona que edita es una sola, los conflictos de verdad
son raros y un numero de version se razona en una linea) ni fusion automatica de documentos.

### El feed es de solo lectura

Los dispositivos escriben por los endpoints normales, cada uno con su idempotencia, y encolan esas
llamadas cuando no hay conexion. No hay un endpoint de "subir lote de cambios": seria una segunda
forma de escribir cada entidad, con su propia validacion que mantener en paralelo. Tampoco hay
empuje: el dispositivo pregunta al arrancar, al volver el foco, despues de sus propias escrituras
y cada 5 minutos.

## Consecuencias

- La normalizacion de remotos es **codigo de dominio compartido y versionado**: vive en
  `git-rudder-core-domain`, la usan el motor y la API, y tiene tests con la tabla de ejemplos.
  Tocarla pide una migracion que recalcule y una las claves que colisionen.
- La tabla de cambios necesita **un contador por ambito** con su cerrojo de fila, y cada escritura
  de dominio tiene que tomar su numero en la misma transaccion que el dato. Un cambio que se
  escriba sin numero es un cambio que ningun dispositivo vera.
- Las lapidas obligan a **una purga programada** a los 90 dias y a que el feed sepa cual fue la
  ultima purgada por ambito para poder contestar 410.
- Los clientes tienen que implementar **dos caminos de lectura**: el incremental y la
  resincronizacion completa con barrido final. El segundo es el que casi nunca corre y el que hay
  que probar a proposito; sin el, el 410 no sirve de nada.
- La cola de escrituras del motor tiene que tratar como respuestas normales el 409 `id_conflict`,
  el 412 y el 409 `sync_category_disabled`: descartar la escritura y avisar, en vez de reintentarla
  para siempre.
- Un canal de avisos en tiempo real (SSE con "hay cambios") queda fuera. Si entra, el cursor y las
  lapidas siguen valiendo igual: solo cambiaria cuando se pregunta, no que se contesta.

## Alternativas descartadas

- **Identificar el repositorio por su ruta local.** Es lo que haria cualquier cliente de un solo
  equipo, y es justo lo que no sirve: la ruta no existe en el otro portatil, y guardarla en la nube
  es filtrar los nombres de los clientes del usuario sin ninguna ganancia.
- **Dejar que el cliente mande el `remoteKey`.** Una linea menos en el servidor, a cambio de que
  dos versiones del cliente que normalicen distinto partan el repositorio en dos. El servidor lo
  recalcula siempre.
- **Un cursor con marca de tiempo**, o una secuencia global. Las dos se pierden cambios, como se
  explica arriba. Una secuencia global es la mas tentadora porque parece ordenada, y es la que
  falla en silencio.
- **Un cursor legible** (`?since=42`). Mas facil de depurar y mucho mas facil de romper: cualquiera
  pasa un numero a mano, y el dia que haga falta meter mas de un contador no se puede cambiar la
  forma sin romper a todos.
- **Filtrar el feed por las categorias de sincronizacion.** Parece lo coherente, y obliga a tener
  un cursor por categoria para poder reanudar una sin rebobinar las demas. Entregar todo es mas
  trafico y un contrato mas simple.
- **Lapidas para siempre.** Resuelve el 410 a cambio de una tabla que solo crece. 90 dias cubre un
  portatil de vacaciones; mas que eso es pagar por el caso de un equipo que estuvo meses apagado,
  que la resincronizacion completa ya resuelve.
- **Fusion automatica de los documentos de ajustes** por campo y marca de tiempo. Es lo que hace
  falta cuando varias personas editan lo mismo; aqui la persona es una sola y el coste no se paga.
