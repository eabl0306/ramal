# ADR 0013: Proxy de IA para las llaves de organizacion, llaves personales en el dispositivo

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01. Las llaves de organizacion y el proxy entran en la primera version publica
  ([ADR 0026](0026-planes-y-precios.md)); el admin fija el modelo por funcion
  ([ADR 0025](0025-recomendacion-de-modelos-de-ia.md)); y se anota donde vive la KEK en produccion
  ([ADR 0023](0023-despliegue-en-apps-01-y-dominio.md))

## Contexto

Ramal genera tres cosas con IA: el mensaje de commit a partir de los cambios preparados, el titulo
y la descripcion de un pull request, y el resumen de un commit en el panel de detalle. El usuario
elige proveedor (Anthropic, OpenAI, Groq, Google, Mistral, OpenRouter, Ollama o cualquier endpoint
compatible con OpenAI), modelo y llave, y cada llave tiene un alcance: personal u organizacion.

Los dos alcances tienen necesidades opuestas:

- Una **llave personal** es del usuario. Ollama vive en el `localhost` de su maquina y ningun
  servidor puede llegar a el. Y si la llave es suya, el codigo no tiene por que pasar por Ramal.
- Una **llave de organizacion** la da de alta un admin para que la usen todos los miembros. Si se
  copiara a cada dispositivo, cualquier miembro podria sacarla y usarla fuera de Ramal, y quitar a
  alguien del equipo no la recuperaria.

La decision de partida del usuario: **las llaves de organizacion nunca salen del servidor**,
cifradas en reposo con cifrado de sobre (AES-256-GCM, llave maestra con identificador para rotar),
y la API hace de proxy de la generacion. **Las llaves personales se usan desde el motor de Rust
del dispositivo**, que es la unica forma de que funcione Ollama. Este ADR la desarrolla.

## Decision

La IA corre donde vive la llave.

### Llaves personales: en el dispositivo

- Se guardan en el llavero del sistema (Desktop) o en `SecretStorage` (VS Code). Si el usuario
  activa su sincronizacion, que viene apagada, viajan por la boveda de extremo a extremo
  ([ADR 0012](0012-boveda-de-extremo-a-extremo.md)).
- El caso de uso del motor arma la entrada, y el adaptador de `ramal-ai` llama al proveedor desde
  el dispositivo. El texto llega a la interfaz en streaming como notificaciones del motor.
- La API no tiene ningun endpoint que genere con una llave personal.

### Llaves de organizacion: cifrado de sobre en el servidor

- **Una llave de datos (DEK) aleatoria de 256 bits por secreto.** Cifra la llave del proveedor
  con AES-256-GCM, nonce aleatorio de 96 bits y datos asociados `ramal:org-ai-key:v1:{orgId}:{keyId}`.
  Un ciphertext copiado a otra fila o a otra organizacion no descifra. Con una DEK por secreto no
  hay que contar cuantos mensajes cifra una llave para no repetir nonce.
- **La DEK se envuelve con la llave maestra activa (KEK)**, tambien con AES-256-GCM y con datos
  asociados `ramal:org-ai-dek:v1:{orgId}:{keyId}:{kekId}`. Se guardan `ciphertext`, `nonce`,
  `wrapped_dek`, `dek_nonce`, `kek_id` y `alg`, mas los metadatos (proveedor, etiqueta, modelo por
  defecto y, si el admin los fija, un modelo por funcion, URL base y la mascara).
- **El `kekId` va en los datos asociados de la envoltura, no en los del secreto.** Si fuera en los
  del secreto, rotar la KEK (que solo reenvuelve la DEK) dejaria todos los secretos sin poder
  descifrarse, porque sus datos asociados ya no coincidirian. Un test rota la KEK y vuelve a
  descifrar cada llave.
- **Las KEK no estan en la base.** Llegan por configuracion (`RAMAL__SECRETS__KEKS`, un mapa de
  identificador a llave en base64, y `RAMAL__SECRETS__ACTIVE_KEK`), se cargan en tipos que se
  limpian de memoria y nunca se registran. Un volcado de la base solo trae ciphertext y DEK
  envueltas. Mas adelante, el mismo puerto `KeyEnvelope` puede tener un adaptador sobre un KMS para
  que la KEK no entre nunca en el proceso.
- **En produccion la KEK vive en una maquina compartida.** Va en el `.env` de Ramal en `apps-01`,
  que tambien sirve otras dos aplicaciones
  ([ADR 0023](0023-despliegue-en-apps-01-y-dominio.md)). La protegen los permisos 0600 del fichero,
  que solo el contenedor de la API la recibe y que cada aplicacion tiene su proyecto de compose, su
  red y sus volumenes. Quien tenga root en la maquina puede leerla, como en cualquier servidor sin
  KMS.
- **Rotar la KEK** es cambiar la activa para las escrituras nuevas. Un trabajo periodico reenvuelve
  las DEK de las filas con la KEK vieja (48 bytes por fila, sin tocar el ciphertext) y la vieja se
  retira cuando ninguna fila la usa. Rotar la llave del proveedor genera una DEK nueva.
- **El alta y la rotacion de estas llaves son los unicos endpoints que reciben en claro un secreto
  de un tercero para guardarlo**, y es por diseno: la llave es de la organizacion y el servidor
  tiene que poder usarla. (Las contrasenas de Ramal y el canje del broker tambien pasan por la API,
  pero ni son de un tercero ni se guardan en claro.) Llega por TLS, se cifra en el acto y el
  valor en claro va en tipos con `Debug` redactado que no aparecen en ningun log ni traza.
- Ningun endpoint devuelve la llave, a nadie, en ningun formato. Los miembros ven proveedor,
  etiqueta, modelo y la mascara. Un admin puede probarla (una llamada minima a la lista de
  modelos): aqui si hay prueba de conexion, porque el servidor tiene la llave.
- Crate: `aes-gcm` 0.11 de RustCrypto, en Rust puro. `aws-lc-rs` necesitaria cmake y un compilador
  de C, igual que las features por defecto de Salvo que ya se apagaron
  ([ADR 0003](0003-rust-con-salvo.md)).

### El proxy

`POST /v1/ai/generate`, solo desde una sesion nativa y con comprobacion de que la sesion no esta
revocada. La web no tiene interfaz de IA y no tiene por que poder gastar llaves.

- **Entrada estructurada, no un prompt libre.** El cliente manda la funcion (`commit_message`,
  `pull_request` o `commit_summary`), el idioma, las opciones (Conventional Commits, instrucciones
  extra) y los datos de entrada (rama y diff preparado, commits de la rama, o un commit). El
  servidor monta el prompt con las plantillas de `ramal-ai`, las mismas que usa el motor con una
  llave personal, asi que el resultado es igual por los dos caminos.
- **La entrada estructurada no impide usar la llave como chat**: las instrucciones extra, la
  plantilla de PR, los cuerpos de commit y los parches son texto libre, y la salida vuelve al
  cliente. Lo que acota el abuso son los limites: un tope de tokens de salida por funcion (400 para
  un mensaje de commit, 1500 para un PR y 600 para un resumen), 500 caracteres de instrucciones
  extra, la cuota reservada antes de llamar y como mucho dos generaciones a la vez por usuario y
  organizacion.
- **El modelo lo fija el admin.** Cada llave de organizacion tiene un modelo por defecto y, si el
  admin quiere, uno por funcion. La peticion no lleva modelo: el proxy usa el de la funcion o, si no
  hay, el de la llave. Un miembro no puede gastar la llave de la organizacion en un modelo mas caro
  que el que eligio el admin ([ADR 0025](0025-recomendacion-de-modelos-de-ia.md)).
- **Respuesta en JSON o en SSE.** Con `Accept: text/event-stream` el texto llega a medida que se
  genera, que es lo que pinta el "Generando..." del diseno. Si el cliente cierra la conexion, la
  API corta la llamada al proveedor.
- **Limites.** 256 KiB de entrada, 60 segundos de espera al proveedor, 20 peticiones por minuto y
  usuario, y la politica de la organizacion: funciones permitidas, peticiones diarias por miembro y
  presupuesto mensual de tokens.
- **La cuota se reserva antes de llamar, en la base.** Con varias replicas, un contador en memoria
  multiplicaria la cuota por el numero de procesos; y comprobar la tabla de consumo y escribir la
  fila despues deja pasar a la vez todas las peticiones paralelas. Por eso cada generacion toma la
  fila de contadores de su organizacion con `FOR UPDATE`, comprueba la cuota diaria del miembro y
  el presupuesto mensual, y deja una fila `pending` con la entrada mas el tope de salida antes de
  llamar al proveedor. Al terminar se liquida con el uso real. Una generacion cancelada se liquida
  con la entrada mas el tope de salida, porque el proveedor cobra la llamada entera aunque nadie
  lea el final. Las peticiones que no llegan al proveedor no dejan fila.
- **Los errores al cliente son categorias** (`unauthorized`, `quota`, `unavailable`), nunca el texto
  del proveedor, que puede traer fragmentos de la peticion.
- **502 y 504 no son un 500.** El cliente tiene que distinguir "Ramal fallo" de "la llave de tu
  organizacion no tiene saldo", porque lo segundo lo arregla un admin.

### Que sale, a donde, y que se guarda

- **Se envia**: el diff de los ficheros preparados o los commits, rutas, estados, nombres de rama,
  mensajes, las instrucciones extra y el idioma. Antes de enviar, el motor quita los ficheros
  sensibles (`.env`, `*.pem`, `*.key` y los patrones del usuario), los binarios, y recorta cada
  parche y el total.
- **La API vuelve a pasar el filtro de ficheros sensibles** y devuelve en `excludedPaths` lo que
  quito. Es defensa en profundidad: un cliente viejo o con un fallo no puede mandar un `.env` al
  proveedor.
- **Se registra** solo organizacion, usuario, llave, funcion, proveedor, modelo, tokens, latencia y
  resultado, 90 dias. La tabla no tiene columna para el contenido.
- **No se guarda ni se registra nunca** la entrada ni la salida: ni en la base, ni en los logs, ni
  en una traza de error. El cuerpo de esta peticion no pasa por el middleware de trazas.
- La interfaz dice todo esto con el nombre del proveedor delante, porque es codigo del usuario
  saliendo de su maquina.

### La URL base es una puerta de SSRF y de fuga

Una URL base que decide un admin sirve para dos ataques: hacer que la API llame a servicios
internos del despliegue, y llevarse la llave de la organizacion apuntandola a un servidor propio,
que recibiria la llave en la cabecera `Authorization`. Las dos cosas romperian la regla de que la
llave no sale del servidor. Por eso:

- **Los proveedores conocidos tienen el host fijo**: `api.openai.com`, `api.anthropic.com`,
  `api.groq.com`, `generativelanguage.googleapis.com`, `api.mistral.ai` y `openrouter.ai`. Su URL
  base no se elige. Cualquier otro host solo con el proveedor `custom`, y `custom` solo si el
  despliegue lo permite (`RAMAL__AI__ALLOW_CUSTOM_BASE_URLS=true`).
- **Cambiar la URL base de una llave `custom` exige reenviar la llave** en la misma peticion, deja
  un evento de auditoria y avisa a la titular de la organizacion. Un admin no puede redirigir una
  llave que dio de alta otra persona sin tenerla.
- **La guarda de SSRF actua al conectar, en cada llamada** (alta, prueba y generacion), no solo al
  guardar: comprobar el DNS una vez no impide que el nombre apunte despues a otra direccion. El
  cliente HTTP usa un resolvedor propio (`reqwest::dns::Resolve`) que descarta las direcciones de
  `0.0.0.0/8`, `10.0.0.0/8`, `100.64.0.0/10`, `127.0.0.0/8`, `169.254.0.0/16`, `172.16.0.0/12`,
  `192.168.0.0/16`, `198.18.0.0/15`, `224.0.0.0/4`, `240.0.0.0/4`, `::1`, `fc00::/7`, `fe80::/10`,
  `::ffff:0:0/96` y `64:ff9b::/96`, y se construye sin redirecciones (`redirect(none)`) y sin proxy
  del entorno (`no_proxy()`). Solo `https`.
- Ollama no se admite como llave de organizacion: el servidor no puede llegar al `localhost` de
  nadie.

### Un crate para los dos caminos

`ramal-ai` (planificado) es una biblioteca neutral que no depende de ningun crate de Ramal
([ADR 0002](0002-arquitectura-hexagonal.md)). La usan `ramal-engine` y `ramal-api-infrastructure`,
y contiene:

- Las plantillas de prompt, versionadas por funcion e idioma, y las reglas de Conventional
  Commits.
- El filtro de ficheros sensibles y el recorte del diff.
- Tres adaptadores de proveedor escritos sobre `reqwest`: chat compatible con OpenAI (que cubre
  OpenAI, Groq, Mistral, OpenRouter, Ollama y `custom`), Messages de Anthropic y Gemini.
- La lista de modelos se pide al endpoint de modelos de cada proveedor y se cachea, con un campo
  manual de respaldo. Nunca va escrita en la app: los modelos del prototipo ya estaban caducados el
  dia del analisis. Que modelo se recomienda sale de un fichero remoto que sirve la API
  ([ADR 0025](0025-recomendacion-de-modelos-de-ia.md)).

## Consecuencias

A favor:

- Una llave de organizacion no se puede sacar de Ramal ni usar fuera de el, y quitar a un miembro
  le quita el acceso de verdad.
- Las organizaciones tienen cuotas y una vista de consumo, asi que la llave no es un cheque en
  blanco.
- Ollama y las llaves personales funcionan sin cuenta de Ramal y sin que el codigo pase por
  nuestros servidores.
- El resultado es el mismo con una llave personal y con una de organizacion: mismas plantillas,
  mismo filtro.

En contra:

- **El proxy hace pasar codigo por Ramal.** Esta acotado (solo llaves de organizacion, entrada
  estructurada, filtro doble, nada guardado), pero cada organizacion tiene que saberlo antes de dar
  de alta una llave.
- **Perder las KEK pierde todas las llaves de organizacion.** Copia fuera de linea obligatoria,
  igual que la llave del actualizador.
- **Hay dos caminos de ejecucion que mantener iguales.** El crate compartido lo reduce a la
  llamada, pero el streaming llega por notificaciones del motor en un caso y por SSE en el otro.
- **La API se convierte en un objetivo para gastar llaves ajenas.** Lo acotan la sesion nativa
  viva, el limite por minuto, la concurrencia por usuario, el tope de salida por funcion y las
  cuotas reservadas en la base.
- **Solo los proveedores de la lista tienen host fijo.** Un proveedor nuevo exige anadirlo al
  codigo; mientras tanto, solo como `custom` y si el despliegue lo permite.
- **Tres adaptadores propios** en lugar de una biblioteca: cada cambio de API de un proveedor es
  trabajo nuestro, y necesitan tests contra respuestas grabadas.

## Alternativas descartadas

**Repartir la llave de organizacion a los dispositivos de los miembros.** Sin servidor en medio,
pero cualquier miembro se la puede llevar, y no hay forma de retirarla al salir del equipo.

**Todo por la API, tambien las llaves personales.** Todo el codigo pasaria por Ramal, Ollama seria
imposible y cada generacion costaria infraestructura.

**Extremo a extremo tambien para las llaves de organizacion.** El servidor no podria llamar al
proveedor, que es todo lo que se le pide.

**Un endpoint con prompt libre.** Mas facil de usar como chat que la entrada estructurada, y con
plantillas distintas en el motor y en la API. La entrada estructurada no elimina ese uso, pero lo
hace mas caro, y los limites hacen el resto.

**`genai` 0.6 como cliente de proveedores.** Cubre muchos proveedores con streaming, pero es
anterior a la 1.0 y cambia a menudo. Seis de los ocho proveedores hablan el protocolo compatible
con OpenAI, asi que tres adaptadores propios cubren todo con menos superficie.

**KMS desde el primer dia.** Mejor aislamiento de la KEK, a cambio de atar el despliegue a un
proveedor de nube antes de saber cual. El puerto `KeyEnvelope` deja la puerta abierta.
