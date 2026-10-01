# ADR 0014: Ramal funciona sin cuenta de Ramal

- Estado: aceptado
- Fecha: 2026-09-30

## Contexto

Ramal es, antes que nada, un cliente Git. Quien lo instala quiere abrir un repositorio y ver su
grafo, no crear una cuenta. Pero los tres productos comparten una cuenta y hay funciones que no
tienen sentido sin ella: sincronizar ajustes y cuentas entre equipos, equipos y organizaciones,
llaves de IA de organizacion, la lista de dispositivos y el panel web.

La decision de partida del usuario: **todo lo que es Git local funciona sin cuenta de Ramal.** La
sincronizacion, los equipos, las llaves de IA de organizacion, los dispositivos y el panel exigen
iniciar sesion. Este ADR fija que significa eso en el codigo y en el contrato, porque es facil de
romper sin querer: basta un caso de uso que pida la sesion "por si acaso", o una pantalla que no
arranque sin red.

## Decision

### La regla

**Nada de Git local mira si hay sesion.** El motor arranca sin red y sin cuenta, y solo los casos
de uso que necesitan la nube preguntan por ella. No hay muro de login: el primer arranque ofrece
"Iniciar sesion" o "Seguir sin cuenta", y las dos opciones llevan a un producto que funciona.

### Que funciona sin cuenta

| Funcion                                                                                   | Sin cuenta | Por que                                                                                                          |
| ----------------------------------------------------------------------------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------- |
| Abrir repositorios, grafo, staging, commit, ramas, merge, rebase, stash, tags, conflictos | Si         | Es el motor contra el git local                                                                                  |
| Fetch, pull, push y clonado                                                               | Si         | Con las credenciales del sistema o con las de una cuenta de plataforma ligada en el equipo                       |
| Cuentas de plataforma con token personal, device flow o PKCE de cliente publico           | Si         | El token va de la plataforma al llavero del equipo ([ADR 0018](0018-cuentas-de-plataformas-git.md))              |
| Conectar Bitbucket por OAuth                                                              | Si         | El broker de la API es publico, con limite por IP                                                                |
| Pull requests, checks y revisiones                                                        | Si         | El motor llama a la plataforma con el token de la cuenta del repositorio                                         |
| IA con llave personal, Ollama incluido                                                    | Si         | El motor llama al proveedor desde el equipo ([ADR 0013](0013-proxy-de-ia-de-organizacion.md))                    |
| Buscar actualizaciones y descargar                                                        | Si         | Los endpoints de publicaciones y del actualizador son publicos ([ADR 0016](0016-actualizaciones-y-descargas.md)) |

### Que pide sesion

| Funcion                                        | Quien lo comprueba                                                                              |
| ---------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| Sincronizar ajustes, workspaces y repositorios | El motor (`CloudClient` sin sesion devuelve `not_signed_in`) y la API (401)                     |
| Sincronizar tokens y llaves personales         | Lo mismo, y ademas la frase de sincronizacion ([ADR 0012](0012-boveda-de-extremo-a-extremo.md)) |
| Equipos, miembros e invitaciones               | La API                                                                                          |
| Llaves de IA de organizacion                   | El motor y la API, que ademas exige ser miembro (404 si no lo es)                               |
| Dispositivos y sesiones activas                | La API                                                                                          |
| Panel web                                      | La guardia de rutas de la web y la API                                                          |

### Como lo cumple el codigo

- **El error tiene nombre.** `not_signed_in` es un valor del `ErrorKind` cerrado del protocolo del
  motor. Solo lo devuelven los casos de uso que usan `CloudClient`, y la interfaz lo traduce como
  cualquier otro.
- **La llamada a iniciar sesion aparece en el punto concreto**, no como un bloqueo: el boton de
  "Sincronizar" o la seccion de equipo ensenan "Inicia sesion para..." y el resto de la app sigue
  funcionando.
- **Ningun arranque espera a la red.** El motor no consulta la API para abrir un repositorio ni
  para pintar nada. Si hay sesion, sincroniza en segundo plano: al arrancar, al volver el foco,
  despues de sus propias escrituras y cada 5 minutos.
- **Los endpoints que un cliente sin cuenta puede necesitar son publicos y no piden identidad**:
  publicaciones y actualizador, y el broker de OAuth de plataformas. Ninguno guarda nada asociado a
  una persona.

### Donde viven los datos sin cuenta

En el equipo, y solo en el equipo:

- **Desktop:** ajustes, lista de repositorios recientes, workspaces locales y metadatos de cuentas
  en el directorio de datos de la app; tokens y llaves en el llavero del sistema.
- **VS Code:** ajustes en la configuracion de VS Code, estado en el almacen de la extension, y
  tokens y llaves en `SecretStorage`.

Las rutas locales, los repositorios sin remoto, las pestanas y los borradores son siempre del
dispositivo, tambien con cuenta: nunca llegan a la API.

### Pasar de local a cuenta, y de vuelta

**Al iniciar sesion por primera vez en un equipo**, para cada categoria de sincronizacion activa:

- Si la cuenta no tiene ese dato en la nube (un documento de preferencias con version 0, ningun
  workspace), el equipo sube el suyo como version inicial. Los ids los genero el cliente, asi que
  un reintento no duplica nada.
- Si la cuenta ya lo tiene, gana la nube y el equipo lo aplica. El usuario que configuro Ramal en
  otra maquina espera encontrar esa configuracion, no que la sobrescriba un equipo recien llegado.
- Las cuentas de plataforma creadas en local se suben como metadatos, y su token solo si hay
  boveda configurada.

**Al cerrar sesion**, el motor borra la llave de la boveda en cache y las copias sincronizadas de
los tokens y llaves que no se crearon en ese equipo. Lo que el usuario creo alli se queda en su
llavero, porque sin cuenta Ramal sigue siendo un cliente Git que necesita esas credenciales para
hacer push. Los repositorios, los ajustes locales y el historial no se tocan. La misma limpieza
corre cuando un refresco recibe un 401 definitivo: un equipo revocado desde otro sitio limpia lo
sincronizado aunque nadie pulse "Cerrar sesion".

### Lo que sale del equipo sin cuenta

- La consulta al actualizador, con un numero de 0 a 99 en una cabecera para el despliegue
  gradual, calculado de una semilla aleatoria que no es el `installationId`
  ([ADR 0016](0016-actualizaciones-y-descargas.md)). No identifica la instalacion, y la busqueda de
  actualizaciones se puede apagar en los ajustes.
- Las llamadas del broker de OAuth si se conecta Bitbucket.
- Las llamadas a las plataformas Git y a los proveedores de IA que el propio usuario configura.

No hay telemetria prevista. Si algun dia la hay, tendra su propio ADR y sera opcional.

## Consecuencias

A favor:

- Ramal se puede usar en una maquina sin red, en un entorno que no deja crear cuentas o
  simplemente por quien no quiere una, y sigue siendo un cliente Git completo.
- La caida de la API no deja a nadie sin hacer commit ni push.
- La cuenta es un anadido con valor claro (sincronizar, equipo, IA de organizacion), no un peaje.
- La regla es facil de comprobar: los casos de uso de Git no dependen de `CloudClient`, y eso se ve
  en sus constructores.

En contra:

- Hay que disenar y probar cada pantalla en dos estados, con y sin sesion.
- El broker publico de Bitbucket deja que cualquiera gaste el cupo del consumer de Ramal. Si se
  abusa, pasa a exigir sesion y el camino sin cuenta queda en el API token.
- La regla de "gana la nube" en el primer inicio de sesion puede descartar ajustes locales que el
  usuario queria conservar. Se acepta por simple; si molesta, el aviso del primer inicio de sesion
  puede ofrecer elegir.
- La consulta del actualizador lleva un numero de 0 a 99 aunque no haya cuenta. No identifica la
  instalacion ni se cruza con nada, pero es un dato que sale del equipo, y por eso la busqueda de
  actualizaciones se puede apagar.

## Alternativas descartadas

**Cuenta obligatoria.** Simplifica el modelo (siempre hay un usuario), a cambio de poner un muro
delante de un cliente Git. Es la queja mas comun contra otros productos del sector.

**Producto solo local, sin nube.** Pierde la sincronizacion entre equipos y todo lo de equipos,
que es lo que justifica la cuenta.

**Broker de OAuth solo con sesion.** Obligaria a crear una cuenta de Ramal para conectar
Bitbucket, que es parte de trabajar con Git. Queda como respuesta si el broker publico se abusa.

**Fusionar campo a campo los ajustes locales con los de la nube.** Mas fiel a lo que el usuario
tenia en cada sitio, pero con reglas de conflicto que nadie entiende. Una persona edita sus
ajustes; "gana la nube" se explica en una frase.
