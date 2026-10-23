# ADR 0026: Gratis para cada persona, Git Rudder Pro por asiento para las organizaciones, todo en la primera version

- Estado: aceptado
- Fecha: 2026-10-01

## Contexto

El diseno ensena una tarjeta "Git Rudder Pro" con asientos, una seccion de equipo, invitaciones y
llaves de IA de organizacion, pero no decia que es gratis, que se paga, a cuanto, ni si todo eso
sale en la primera version publica. Eran dos decisiones abiertas de
[gaps.md](../design/gaps.md#4-decisiones-abiertas-para-ezequiel): Q-01 (equipos y facturacion en
la primera version o despues) y Q-03 (que es gratis, que es Git Rudder Pro y cuanto cuesta el asiento).
Hasta ahora el backlog suponia que equipos, IA de organizacion y facturacion iban despues del MVP.

Lo que ya estaba fijado y no se toca:

- **Todo lo que es Git local funciona sin cuenta** ([ADR 0014](0014-usar-git-rudder-sin-cuenta.md)).
- La licencia es la FSL, que impide clones comerciales durante dos anos por version
  ([ADR 0021](0021-licencia-fsl.md)).
- El contrato ya pone el plan en la organizacion: la tarjeta de Desktop cuelga de ella.

Como cobra la competencia (consultado el 2026-10-01): GitKraken cobra por asiento y su plan
gratuito no admite repositorios privados (Pro unos 10 USD al mes en pago anual, el plan de equipo
14); Tower, de unos 6 a 11 USD al mes por usuario segun el plan; SmartGit es gratis para uso no
comercial; Sublime Merge y Fork, pago unico; GitButler da el cliente gratis con la misma licencia
que Git Rudder y cobra por las funciones de equipo. Los que cobran por el cliente rondan de 5 a 11 USD al
mes; los que dan el cliente gratis cobran por el equipo y la nube.

## Decision

### Todo sale en la primera version publica

Organizaciones, equipo, invitaciones, asientos, facturacion **y** llaves de IA de organizacion con
el proxy de la API salen en la primera version publica. En el backlog, las epicas E27, E28, E29 y
E30 pasan de Post-MVP a MVP, y en gaps.md las filas W-11 a W-18 y D-47 son del MVP.

### Pagan los equipos

| Que                                                                       | Gratis, con o sin cuenta  | Git Rudder Pro |
| ------------------------------------------------------------------------- | ------------------------- | -------------- |
| Todo Git local, repositorios privados incluidos                           | Si                        | Si             |
| Pull requests de GitHub, GitLab, Bitbucket, Gitea y Azure DevOps          | Si                        | Si             |
| Cuentas de plataforma sin limite                                          | Si                        | Si             |
| IA con llave propia, Ollama incluido                                      | Si                        | Si             |
| Sincronizacion personal de extremo a extremo, dispositivos y panel web    | Si (pide cuenta, no pago) | Si             |
| Organizacion, invitaciones, roles y asientos                              | No                        | Si             |
| Workspaces compartidos con el equipo                                      | No                        | Si             |
| Llaves de IA de organizacion por el proxy, con politica, cuotas y consumo | No                        | Si             |
| Facturacion centralizada                                                  | No                        | Si             |

- **Git Rudder Pro es el plan de una organizacion, por asiento.** No hay plan personal de pago: una
  persona sola usa todo gratis, repositorios privados y sincronizacion incluidos.
- **Precio:** 8 USD por asiento al mes en pago anual (96 USD al ano) o 10 USD en pago mensual. La
  moneda base es el dolar; el proveedor de pagos ensena el precio en la moneda del cliente
  ([ADR 0022](0022-proveedor-de-pagos.md)).
- **Prueba de 14 dias sin tarjeta** al crear una organizacion.
- **Minimo un asiento.** Las invitaciones pendientes ocupan asiento, como ya decia el contrato.
- **No hay plan gratuito para organizaciones pequenas**: solo la prueba.

Por que 8 USD: queda por debajo de los planes de equipo de GitKraken y de Tower, cosa logica en un
producto nuevo, y por encima de los 5 USD que lo pondrian en el precio de una herramienta personal.
Subir un precio despues cuesta mas que bajarlo con un descuento.

Por que pagan los equipos y no las personas: encaja con el diseno y con el contrato, cobra donde
esta el dinero de los clientes Git (las empresas) y no toca la promesa de que lo local es gratis.
La sincronizacion personal cuesta casi nada de servir y trae cuentas, que son las que despues
invitan a su equipo. Frente a GitKraken, el mensaje es que los repositorios privados son gratis.

### Lo que todavia no esta decidido

No bloquea nada del MVP:

- Un descuento de lanzamiento.
- Git Rudder Pro gratis para proyectos de codigo abierto y educacion, a peticion.
- Precios regionales.
- Que pasa cuando acaba la prueba sin pagar o cuando un pago falla. La propuesta es que la
  organizacion quede en solo lectura y sus datos se conserven 30 dias; lo fija la issue de reglas
  de facturacion (E30-01) antes de construir los limites del plan.

## Consecuencias

A favor:

- Hay ingresos desde el lanzamiento, y el producto sale como lo ensena el diseno, con la tarjeta de
  plan visible desde el primer dia.
- Un mensaje simple: gratis para cada persona, de pago para el equipo.
- Los limites del plan son pocos y del lado de la organizacion: las funciones de organizacion solo
  funcionan con una suscripcion activa o en prueba. No hay topes que mantener en lo personal.

En contra:

- **El MVP crece unas 44 issues** (las cuatro epicas), alrededor de un 11 % sobre las 410 que tenia,
  y el lanzamiento espera a la facturacion y a la aprobacion del proveedor de pagos.
- **La primera version estrena a la vez el cobro y la parte mas delicada del producto**: el proxy
  de IA, el unico sitio donde el servidor guarda secretos de terceros y por donde pasa codigo del
  usuario ([ADR 0013](0013-proxy-de-ia-de-organizacion.md)).
- Un usuario suelto no paga nunca. Los ingresos dependen de que lleguen equipos.
- En el pago mensual, la comision fija del proveedor se lleva alrededor del 10 %. Conviene empujar
  el anual.
- Una persona puede encadenar pruebas creando organizaciones. Lo acota el tope de cinco
  organizaciones por usuario; si se abusa, la prueba pasa a ser una por titular.

## Alternativas descartadas

**Lanzar sin equipos y cobrar despues.** Se lanza antes y lo mas delicado no entra en la primera
version, pero sin ingresos al lanzar y con la tarjeta de Git Rudder Pro escondida.

**Equipos y cobro en la primera version, IA de organizacion despues.** Menos riesgo en la primera
version, pero el argumento de venta de los equipos queda mas flojo sin las llaves de organizacion.

**Equipos gratis en beta y cobro despues.** Da opiniones de equipos pronto, pero regalar algo y
cobrarlo despues genera quejas, y obliga a decidir que pasa con esas organizaciones.

**Un plan personal de pago** que desbloquee la sincronizacion entre equipos. Ingresos antes de
tener equipos, pero sincronizar ajustes es lo que la gente espera gratis, y obliga a montar una
suscripcion personal y limites en el motor, la API y las apps por poco dinero.

**Gratis con topes en la nube** (dispositivos o workspaces). Los topes se sienten mezquinos y hay
que mantenerlos en tres productos.

**Gratis solo para uso no comercial**, como SmartGit. Choca con "todo Git local gratis" y obligaria
a una licencia o una cuenta para lo local.

**Organizaciones pequenas gratis** (por ejemplo, dos asientos). Los equipos pequenos son los
primeros clientes probables; regalarles el plan deja sin ingresos al segmento que mas va a probar
el producto.
