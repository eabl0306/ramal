# ADR 0018: Cuentas de plataformas Git, clientes publicos primero

- Estado: aceptado
- Fecha: 2026-09-30

## Contexto

El diseno deja conectar varias cuentas por plataforma (GitHub, GitLab, Bitbucket, Gitea y Azure
DevOps), con usuario, etiqueta, host, metodo, scopes, "Usada en N repos" y una predeterminada por
plataforma. Cada repositorio remoto usa una de ellas para fetch, push y pull requests. Los dialogos
de clonar y de crear un PR eligen cuenta. Los PRs aparecen en las tablas de los workspaces (estado,
checks, revisiones) y en la barra lateral del repositorio.

Tres decisiones anteriores lo condicionan:

- **Los tokens son de extremo a extremo** ([ADR 0012](0012-boveda-de-extremo-a-extremo.md)): el
  servidor no puede descifrarlos, no hay cuentas compartidas con el equipo y la web no puede probar
  una conexion.
- **Ramal funciona sin cuenta** ([ADR 0014](0014-usar-ramal-sin-cuenta.md)): conectar una
  plataforma no puede exigir una cuenta de Ramal.
- **Las credenciales se eligen por repositorio** con el helper de credenciales y `core.sshCommand`
  ([ADR 0009](0009-motor-git-hibrido.md)).

Y la realidad de cada proveedor en septiembre de 2026, que el diseno no recogia:

- **Bitbucket dejo de admitir app passwords el 9 de junio de 2026.** Los datos del prototipo las
  usaban. Lo que queda son los API tokens (con scopes y caducidad) y OAuth 2.0 con consumer, que
  exige un secreto de cliente para canjear el codigo y no soporta PKCE.
- **Azure DevOps retira su OAuth en 2026** (no admite aplicaciones nuevas desde abril de 2025) y
  **los PAT globales dejan de funcionar el 1 de diciembre de 2026.** Lo que queda son los PAT de
  organizacion y, mas adelante, OAuth de Entra ID, que segun la experiencia publicada de SmartGit no
  cubre las cuentas personales de Microsoft.
- **GitHub** permite device flow en una OAuth App con solo el `client_id`. Una GitHub App solo llega
  a los repositorios donde esta instalada, que es una barrera excesiva para un cliente Git que
  tiene que poder clonar cualquier repositorio al que el usuario tenga acceso.
- **GitLab.com** admite Authorization Code con PKCE para aplicaciones no confidenciales, sin
  secreto. Cada instancia propia de GitLab, Gitea o Forgejo necesitaria su propia aplicacion OAuth
  registrada.

El analisis tecnico proponia que la API hiciera de broker OAuth para todos y guardara los tokens
cifrados. Con la decision de extremo a extremo eso ya no vale: el token pasaria por el servidor.

## Decision

### Varias cuentas por plataforma, una por repositorio

Una cuenta tiene plataforma, host, usuario, etiqueta, metodo, scopes y si es la predeterminada de
su plataforma. El host va en minusculas y sin esquema; en Azure DevOps incluye la organizacion
(`dev.azure.com/banca`), porque alli la organizacion es parte de a donde se conecta. Cada
repositorio queda ligado a una cuenta, y el motor usa sus credenciales para la red de ese
repositorio y para sus llamadas a la API de la plataforma.

**Las credenciales se eligen por host exacto, nunca por plataforma.** Un miembro de una
organizacion puede anadir a un workspace compartido `https://github.com.evil.example/...`, o un host
con letras de otro alfabeto que se parecen, y declarar que es GitHub. Si el motor eligiera la cuenta
por plataforma, el companero que lo clonara entregaria su token de GitHub a ese servidor. Por eso:

- El helper de credenciales solo responde si el protocolo es `https` y el host que pide git es
  identico, despues de normalizarlo con IDNA, al `host` de la cuenta ligada (en Azure DevOps,
  ademas, la ruta empieza por la organizacion de la cuenta). Si no coincide, no entrega nada.
- El dialogo de clonar solo ofrece las cuentas cuyo host coincide con el de la URL.
- La plataforma que declare un cliente para un host propio solo sirve para pintar el icono.
- Los hosts que no son ASCII se ensenan en punycode.
- `PlatformClient` construye la URL base de la API de la plataforma desde el host de la cuenta,
  nunca desde la entrada del repositorio o del workspace.

### Como se conecta cada plataforma

| Plataforma                    | Metodo por defecto                                                               | Tambien                                                   | Pasa por la API de Ramal               |
| ----------------------------- | -------------------------------------------------------------------------------- | --------------------------------------------------------- | -------------------------------------- |
| GitHub (github.com)           | Device flow de la OAuth App, solo con `client_id`                                | PAT de grano fino                                         | No                                     |
| GitHub Enterprise Server      | PAT                                                                              | OAuth App que registre el cliente, despues del MVP        | No                                     |
| GitLab.com                    | Authorization Code con PKCE, aplicacion no confidencial, redireccion de loopback | PAT                                                       | No                                     |
| GitLab propio, Gitea, Forgejo | PAT                                                                              | PKCE si el admin registra una aplicacion, despues del MVP | No                                     |
| Azure DevOps                  | PAT de organizacion                                                              | OAuth de Entra ID mas adelante                            | No                                     |
| Bitbucket Cloud               | OAuth 2.0 con consumer, a traves del broker                                      | API token                                                 | Solo el canje del codigo y el refresco |
| SSH (cualquiera)              | Llave por cuenta para el transporte                                              | Token aparte para la API (metodo `ssh_token`)             | No                                     |

### Clientes publicos primero

Donde el proveedor lo permite, el motor es un cliente OAuth publico: device flow o PKCE con
redireccion de loopback. **El token va de la plataforma al dispositivo y nunca pasa por Ramal.**

- Los `client_id` van compilados en el motor, para poder conectar una cuenta aunque la API no
  responda. `GET /v1/platform-oauth/providers` solo los sobreescribe, para poder rotarlos sin
  publicar una version.
- Los scopes son los minimos para fetch, push, PRs, checks y revisiones (en GitHub, `repo`,
  `read:org` y `workflow`).
- La OAuth App de GitHub se registra sin caducidad de tokens mientras no se compruebe si refrescar
  un token caducado exige el secreto de cliente. Si lo exige, ese refresco pasara por el broker como
  el de Bitbucket.

### El broker, solo donde hace falta un secreto

Una app de escritorio no puede llevar un secreto de cliente dentro del binario: se extrae en
minutos. Para Bitbucket, la API pone el secreto en el canje y no guarda el token:

1. El motor genera un verificador propio y su reto S256, abre un receptor de loopback y lanza el
   navegador a la API, que guarda el flujo y redirige a Bitbucket con su propio `state`. Solo
   acepta redirecciones de loopback, validadas con el mismo analizador estricto que el inicio de
   sesion de las apps ([ADR 0008](0008-autenticacion-con-tokens.md)): esquema `http`, host
   exactamente `127.0.0.1` o `[::1]`, puerto explicito, ruta `/callback` y nada mas. Con una
   comprobacion de prefijo, `http://127.0.0.1:1@evil.example/callback` mandaria el codigo de
   Bitbucket de la victima a otro servidor.
2. Bitbucket vuelve a la API, que **no canjea todavia**: redirige al loopback del motor con el
   codigo y el id del flujo. El navegador aterriza en el receptor del motor, que sirve la pagina
   "Cuenta conectada, vuelve a Ramal" (o el error del proveedor). Si el flujo no existe o caduco,
   la API no tiene a donde redirigir y manda al navegador a una pagina de error de la web.
3. El motor canjea en la API con su verificador. La API comprueba que casa con el reto, canjea con
   Bitbucket poniendo el secreto, devuelve los tokens en la respuesta y borra el flujo.

El reto propio es PKCE entre el motor y la API donde el proveedor no lo da: sin el, cualquiera que
viera un codigo de autorizacion podria canjearlo a traves del broker. El token pasa por la memoria
del proceso durante el canje y nada mas: no se escribe en la base, no se registra (el middleware de
trazas no registra ni la query ni el cuerpo) y la respuesta lleva `Cache-Control: no-store`. Es el
unico sitio de la API por donde pasa un token de plataforma, y vive aislado en su propio modulo.

El broker es publico, sin sesion de Ramal, con limite por IP. Los tokens de Bitbucket caducan a las
2 horas y el refresco va por el mismo broker.

### Tokens personales donde no hay OAuth

En GitLab propio, Gitea, Forgejo, GitHub Enterprise Server y Azure DevOps, el usuario pega un token
personal. Al anadirlo, el motor lo prueba contra la plataforma, lee el usuario y los scopes, y
guarda la caducidad si la plataforma la da. Un PAT de Azure DevOps dura como mucho un ano y es de
una organizacion: la interfaz avisa con tiempo de que caduca.

### Pull requests, checks y revisiones: directo desde el motor

`ramal-platforms` (planificado) implementa el puerto `PlatformClient` para las cinco plataformas y
el motor llama a cada una con el token de la cuenta del repositorio, con una cache local. **La API
de Ramal no ve ningun pull request.** El panel web no ensena PRs, asi que no hace falta proxy.

- Latencia minima, y sigue funcionando si la API de Ramal cae.
- Los limites de peticiones de cada plataforma son por token de usuario de todos modos.
- Ni el codigo ni los metadatos de los PRs pasan por nuestros servidores.

### Lo que sincroniza la nube

- **Metadatos** (plataforma, host, usuario, etiqueta, metodo, scopes, predeterminada, en cuantos
  repositorios se usa), si la categoria "Cuentas vinculadas" esta activa.
- **El token**, sellado en la boveda, solo si ademas hay boveda configurada, y junto a una copia
  sellada de plataforma, host, usuario y metodo. El motor que lo abre compara esa copia con los
  metadatos en claro y no usa el token si no coinciden: los metadatos en claro los controla el
  servidor ([ADR 0012](0012-boveda-de-extremo-a-extremo.md)). Una cuenta sincronizada sin secreto
  aparece en otro equipo con "Actualizar credenciales".
- **El estado** (valida, caducada) y la ultima verificacion **los informa el dispositivo**, que es
  quien puede probar el token. El servidor no los calcula.
- **Desconectar** desde un dispositivo revoca primero el permiso en la plataforma, si la plataforma
  lo permite, y luego borra la cuenta en la nube. Desde la web solo se borran los metadatos y el
  blob, y la web avisa de que el token sigue valido en la plataforma hasta que se revoque alli.
- Las llaves SSH viven en el equipo y nunca se sincronizan.

## Consecuencias

A favor:

- Ningun token de plataforma toca el servidor de Ramal, salvo el canje de Bitbucket, que pasa por
  memoria y se va.
- Conectar GitHub, GitLab.com y las plataformas con PAT funciona sin cuenta de Ramal y sin que la
  API este levantada.
- Varias cuentas de la misma plataforma conviven repositorio a repositorio.
- Nada depende de metodos que los proveedores ya retiraron (app passwords de Bitbucket, OAuth de
  Azure DevOps, PAT globales de Azure).

En contra:

- **El broker publico permite gastar el cupo del consumer de Bitbucket de Ramal a cualquiera.** Si
  se abusa, pasa a exigir sesion de Ramal y el camino sin cuenta queda en el API token. Tambien
  permite usar un refresh token robado de nuestro consumer; quien lo tiene ya tiene acceso a esa
  cuenta hasta que caduque.
- **Las organizaciones de GitHub pueden bloquear la OAuth App** hasta que un admin la apruebe. La
  interfaz tiene que explicarlo con el enlace de aprobacion.
- **En las plataformas con PAT, el usuario crea el token a mano**, con los scopes que le digamos.
  Es peor experiencia que un boton de "Conectar".
- **Los proveedores cambian rapido.** Los metodos de cada plataforma tienen que poder cambiar sin
  ceremonia: en el esquema son texto con `CHECK`, no un `ENUM`, y los `client_id` se pueden rotar
  desde la API.
- **No hay cuentas compartidas con el equipo.** Es la consecuencia aceptada de la boveda.
- Cinco adaptadores de plataforma con sus particularidades (paginacion, limites, formatos de
  checks), cada uno con tests contra respuestas grabadas.

## Alternativas descartadas

**Broker OAuth para todas las plataformas, con los tokens cifrados en el servidor.** Era la
propuesta del analisis. Rompe la decision de extremo a extremo y obliga a tener cuenta de Ramal
para conectar cualquier plataforma.

**GitHub App en lugar de OAuth App.** Solo llega a los repositorios donde esta instalada, y un
cliente Git tiene que poder clonar cualquiera.

**Proxy de PRs, checks y revisiones a traves de la API.** Convierte el servidor en el objetivo que
descifra todos los tokens en cada peticion, anade latencia y coste, y falla cuando falla la API.
Ademas es imposible con la boveda.

**Solo Git Credential Manager y los helpers del sistema.** No sabe elegir la cuenta segun el
repositorio y no da token para las APIs de las plataformas.

**Secretos de cliente dentro del binario.** Se extraen en minutos y obligan a rotarlos en todas las
instalaciones a la vez.

**App passwords de Bitbucket y OAuth de Azure DevOps.** Ya retirados o a punto de estarlo.

**Cuentas compartidas con el equipo.** Descartadas por la decision de extremo a extremo, no
pendientes.
