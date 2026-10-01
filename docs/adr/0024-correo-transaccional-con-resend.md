# ADR 0024: Correo transaccional por SMTP con Resend

- Estado: aceptado
- Fecha: 2026-10-01

## Contexto

La API manda correo desde la primera fase de identidad: verificacion del correo, restablecer la
contrasena, cambio de correo y su enlace de deshacer, avisos de seguridad (dispositivo nuevo,
cambios de la boveda, primera descarga de la boveda en un equipo), enlaces de reautenticacion e
invitaciones a una organizacion ([api-surface.md](../api-surface.md#avisos-de-seguridad)). Con los
equipos y la facturacion en el MVP, las invitaciones tambien salen desde el primer dia.

Lo que ya esta decidido o medido:

- La API envia por **SMTP con `lettre`**, que ya figura en la lista de crates de
  `ramal-api-infrastructure` y en la de prohibidos del nucleo. En desarrollo el buzon es Mailpit,
  que ya esta en `compose.yaml`. Con SMTP, cambiar de proveedor es cambiar la configuracion.
- El volumen es bajo: de dos a cinco correos por usuario el primer mes y casi nada despues. Con
  1.000 altas al mes, unos 3.000 a 5.000 correos. El riesgo no es el total sino el pico del dia
  de lanzamiento.
- Gmail, Yahoo y Microsoft rechazan o mandan a spam el correo sin SPF, DKIM y DMARC alineados.
- Gastonomo ya usa Resend por SMTP y dejo dos lecciones escritas: **el dominio del remitente se
  verifica exacto** (tener verificado `bahoque.com` no verifica `mail.bahoque.com`, y el primer
  envio se rechazo con un 550 por eso) y el puerto 465 no funciono con su transporte; el que
  funciona es el 587 con STARTTLS.
- El dominio de Ramal es `ramal.bahoque.com` ([ADR 0023](0023-despliegue-en-apps-01-y-dominio.md)).

## Decision

**Resend, por SMTP, en un equipo de Resend aparte del de Gastonomo.**

- **Equipo propio** para que Ramal no comparta cuota, llaves ni factura con Gastonomo, y para que
  un problema de reputacion de uno no toque al otro.
- **Plan gratuito durante la beta privada** (3.000 correos al mes, 100 al dia) y **Pro (20 USD al
  mes, 50.000 correos, sin tope diario) al abrir la beta publica**. El tope de 100 al dia es lo
  unico que puede bloquear altas en un lanzamiento, y Pro lo quita.
- **SMTP con `lettre`** en el puerto 587 con STARTTLS. La configuracion es
  `RAMAL__MAIL__{HOST,PORT,USERNAME,PASSWORD,FROM,STARTTLS}`. Las claves van comentadas en
  `.env.example` hasta que la API las lea, porque una clave desconocida hace fallar el arranque.
  La contrasena SMTP (la llave de API de Resend) es un secreto de Actions que escribe el
  despliegue.
- **Mailpit en desarrollo y en los E2E de la pila.** Los tests leen el codigo de verificacion por
  la API de Mailpit; nunca se manda un correo de verdad desde la CI.

### Remitente y registros

- **Remitente en un subdominio de `ramal.bahoque.com`, verificado exacto en Resend**, por ejemplo
  `Ramal <no-reply@mail.ramal.bahoque.com>`. Se verifica ese nombre, no `bahoque.com` ni
  `ramal.bahoque.com`.
- Resend da los registros para ese subdominio (DKIM, y MX y SPF para el dominio de rebote). Los
  valores salen de su panel y se crean en Route 53.
- **DMARC en el subdominio exacto del remitente** (`_dmarc.mail.ramal.bahoque.com`), por etapas:
  `p=none` con informes, despues `p=quarantine` tras dos a cuatro semanas de informes limpios, y
  despues `p=reject`. Va en el subdominio y no en `bahoque.com` para no cambiar la politica del
  dominio que usan Gastonomo y el resto: los receptores miran primero el registro del dominio
  exacto del remitente.
- El correo lleva `Reply-To` a un buzon real. Cual es, y que buzon recibe los informes de DMARC,
  esta sin decidir (ver Consecuencias).
- Alta del dominio en Google Postmaster Tools para ver la reputacion.

## Consecuencias

A favor:

- Patron conocido: es el mismo proveedor y el mismo transporte que Gastonomo, con sus errores ya
  pagados.
- El coste es cero durante la beta privada y 20 USD al mes desde la beta publica.
- Cambiar de proveedor es cambiar seis variables y los registros DNS. Si Postmaster Tools ensena
  problemas de entrega, el plan B es Postmark.

En contra:

- **El plan gratuito tiene un tope de 100 correos al dia.** Durante la beta privada hay que vigilar
  que no se alcance; la beta publica no abre sin Pro.
- Resend envia sobre IPs compartidas de Amazon SES, y los datos de la cuenta se guardan en EE. UU.
  La politica de privacidad tiene que nombrarlo como encargado del tratamiento.
- **Falta decidir que buzon real recibe las respuestas y los informes de DMARC.** Sin el,
  `Reply-To` no se puede fijar y DMARC no puede pasar de `p=none`. Si ese buzon esta en otro
  dominio, ese dominio tiene que autorizar la recepcion de informes de `mail.ramal.bahoque.com`.
- Hay que mantener a mano el calendario de DMARC (`none`, `quarantine`, `reject`).

## Alternativas descartadas

**Amazon SES directo.** Lo mas barato (0,10 USD por cada 1.000 correos) y en la misma cuenta de AWS,
pero hay que pedir salir del sandbox, montar a mano los rebotes y las quejas por SNS, y el panel y
los logs son minimos. Mas trabajo para una persona sola.

**Postmark.** La mejor fama en correo transaccional, pero se paga desde el primer dia, el alta pasa
por revision manual y el exceso es caro. Queda como plan B.

**Brevo.** Plan gratuito con mas margen diario y empresa de la UE, pero pensado sobre todo para
marketing y con entrega mas irregular en IPs compartidas.

**SendGrid.** Ya no tiene plan gratuito.
