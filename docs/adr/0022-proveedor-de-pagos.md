# ADR 0022: Paddle como proveedor de pagos, detras de un puerto `BillingProvider`

- Estado: aceptado
- Fecha: 2026-10-01

## Contexto

Git Rudder cobra desde la primera version publica: las organizaciones, el equipo, los asientos y la
facturacion entran en el MVP, y Git Rudder Pro es un plan por asiento de una organizacion
([ADR 0026](0026-planes-y-precios.md)). Hay que elegir con quien se cobra.

Lo que condiciona la eleccion:

- **Git Rudder lo lleva una sola persona** y vende a clientes de cualquier pais. Vender software por
  suscripcion obliga a cobrar el IVA de la UE desde el primer euro vendido a un particular, el
  IVA del Reino Unido y el impuesto de ventas de cada estado de EE. UU. donde se pase el umbral.
- Hay dos familias de proveedores. Un **comerciante registrado** (merchant of record, MoR) es el
  vendedor ante el cliente: cobra, calcula, declara y paga los impuestos de cada pais, y a Git Rudder
  le paga lo que queda. Un **procesador** (Stripe con Stripe Billing y Stripe Tax) solo mueve el
  dinero: el vendedor es Git Rudder y declarar es cosa suya.
- **El pais desde el que factura Ezequiel no esta confirmado.** Se confirma al abrir la cuenta
  del proveedor. La eleccion tiene que valer para cualquier pais no sancionado.
- El contrato ya tenia la forma de la facturacion (checkout, asientos con prorrateo, portal del
  cliente y webhooks) y la tabla de suscripciones, sin proveedor.

Datos consultados el 2026-10-01:

| Proveedor               | Tipo                            | Comision publicada                                                             | Desde que paises vende                                                                                      |
| ----------------------- | ------------------------------- | ------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------- |
| Paddle                  | MoR                             | 5 % + 0,50 USD, todo incluido                                                  | Cualquiera salvo los sancionados; acepta personas fisicas                                                   |
| Polar                   | MoR sobre Stripe Connect        | 5 % + 0,50 USD, mas 1,5 % con tarjeta no estadounidense                        | Lista amplia de paises de pago                                                                              |
| Lemon Squeezy           | MoR, de Stripe desde 2024       | 5 % + 0,50 USD, con recargos internacionales y de renovacion                   | Amplia, pero su equipo prepara la migracion a Stripe Managed Payments                                       |
| Stripe Managed Payments | MoR de Stripe                   | 3,5 % ademas de las comisiones de Stripe                                       | Solo empresas de EE. UU., Canada, UE y EEE, Reino Unido, Suiza, Australia, Hong Kong, Japon y Singapur      |
| Stripe Billing y Tax    | Procesador; el vendedor eres tu | 2,9 % + 0,30 USD, mas recargos internacionales, de cambio, de Billing y de Tax | Solo paises donde opera Stripe; en Latinoamerica, Mexico y Brasil. Desde otros, con una sociedad en EE. UU. |

## Decision

**Paddle como comerciante registrado. Polar es el plan B.** Los dos van detras del mismo puerto,
asi que cambiar de proveedor es escribir otro adaptador.

### Por que Paddle

- Es la unica opcion que vale igual desde cualquier pais no sancionado, como persona fisica o como
  empresa. La recomendacion no depende del pais que se confirme.
- Asume el IVA y los impuestos de ventas de todo el mundo. Una persona sola no tiene que darse de
  alta a efectos fiscales en ningun pais extranjero ni presentar declaraciones trimestrales fuera
  del suyo.
- Trae lo que pide el contrato: precio por asiento, cambio de asientos con prorrateo, portal del
  cliente para metodo de pago, facturas y cancelacion, y webhooks firmados.
- Precio cerrado, sin recargo internacional publicado. De un asiento anual de 96 USD quedan unos
  90,70.

### El puerto `BillingProvider`

Vive en `git-rudder-api-application`, como cualquier otro puerto
([ADR 0002](0002-arquitectura-hexagonal.md)). El nucleo habla de organizaciones, planes,
asientos y estados de suscripcion, nunca de Paddle:

- `checkout_session(organizacion, plan, intervalo, asientos)`: devuelve lo que la web necesita
  para abrir el pago.
- `change_seats(suscripcion, cantidad)`: cambia los asientos con prorrateo; puede pedir que el
  cliente confirme el pago.
- `portal_session(cliente)`: devuelve la URL del portal del proveedor.
- `verify_webhook(cabeceras, cuerpo)`: comprueba la firma y traduce el evento a un hecho del
  dominio (suscripcion creada, actualizada, con pago atrasado, cancelada).

`PaddleBillingProvider` lo implementa en `git-rudder-api-infrastructure` sobre la API de Paddle Billing
con `reqwest`. Un adaptador falso sirve a los tests de los casos de uso. Polar, o Stripe el dia que
compense, seria otro adaptador del mismo puerto.

### Como se cobra

- **Webhook en `/v1/billing/webhooks/paddle`**, publico. Verifica la firma de Paddle con el
  secreto del webhook (que llega por configuracion), rechaza eventos con una marca de tiempo
  vieja, deduplica por el identificador del evento y descarta un evento mas antiguo que el ultimo
  aplicado a esa suscripcion, porque los webhooks pueden llegar desordenados. La fuente de verdad
  del numero de asientos y del estado es Paddle; la API guarda la copia que le llega por aqui.
- **El checkout es una pagina de la web.** La API devuelve una URL de
  `https://git-rudder.com` que carga Paddle.js y abre el pago de Paddle. **La CSP de la web se
  abre a los dominios de Paddle solo en esa pagina**: el resto de la web, con las paginas de
  autorizar una app y de activar un codigo, sigue sin cargar codigo de terceros. Desktop y
  VS Code abren esa pagina en el navegador del sistema y no tocan su CSP. La API nunca ve una
  tarjeta.
- El checkout lleva el id de la organizacion en los datos propios de la transaccion. Asi el
  webhook sabe a que organizacion pertenece la suscripcion sin adivinarlo por el correo.
- **Los ids de Paddle se guardan en la organizacion**: el del cliente y el de la suscripcion van en
  la fila de `subscriptions`, que es una por organizacion y tiene `organization_id` como clave
  primaria ([data-model.md](../data-model.md#f5-facturacion)).
- **La prueba de 14 dias la lleva Git Rudder, no Paddle.** Al crear una organizacion la API apunta
  cuando acaba la prueba; mientras dura no hay nada en Paddle ni se pide tarjeta. Paddle entra en
  el checkout. Asi la prueba sin tarjeta no depende de lo que permita cada proveedor.
- **E2E contra el sandbox de Paddle**, con sus tarjetas de prueba y un webhook de sandbox. Las
  credenciales del sandbox son secretos de la CI, separados de los de produccion.

### Lo que hay que hacer antes de cobrar

Paddle revisa el dominio y el producto antes de activar los pagos. Pide una landing con precios,
terminos del servicio, politica de privacidad y politica de reembolso publicados en
`git-rudder.com`. La aprobacion puede tardar dias, asi que la cuenta se abre en cuanto esas
paginas existan. Al abrirla se confirma el pais y la forma (persona fisica o empresa) desde la que
factura Ezequiel; segun el pais puede haber tramites locales (por ejemplo, inscribirse como
exportador de servicios), que se consultan con un asesor fiscal y no cambian esta decision.

## Consecuencias

A favor:

- Ningun alta fiscal en otros paises y ninguna declaracion de IVA extranjera, desde la primera
  venta.
- El contrato de facturacion no cambia de forma: checkout, asientos, portal y webhooks ya estaban.
- El nucleo no sabe quien cobra. Cambiar de proveedor es un adaptador y una migracion de datos, no
  un rediseno.
- El sandbox permite probar el ciclo entero (pago, cambio de asientos, cancelacion, pago fallido)
  en la CI.

En contra:

- **Comision alta en tickets pequenos.** En el pago mensual de 10 USD el fijo de 0,50 pesa: Paddle
  se queda alrededor del 10 %. Conviene empujar el pago anual.
- **Paddle aparece delante del cliente**: en el checkout, en el cargo de la tarjeta y en las
  facturas.
- **Mover suscripciones entre comerciantes registrados no es trivial.** Las tarjetas guardadas no
  viajan de un MoR a otro; cambiar de proveedor obliga a que cada cliente vuelva a pagar en el
  nuevo. El puerto abarata el codigo, no esa migracion.
- La pagina de checkout carga codigo de un tercero en el origen de la web. Por eso es una pagina
  aparte y la CSP solo se abre ahi.
- La aprobacion de Paddle depende de paginas legales que todavia no existen, y las piden antes del
  primer cobro.

## Alternativas descartadas

**Stripe Billing con Stripe Tax (vender directamente).** Lo mas barato en cobros domesticos de
EE. UU. y lo mas flexible, pero el vendedor es Git Rudder: Stripe Tax calcula, y declarar y pagar el IVA
de la UE, el del Reino Unido y el impuesto de ventas de EE. UU. es trabajo propio desde la primera
venta a un particular. Ademas Stripe solo opera en algunos paises; desde muchos otros haria falta
una sociedad en EE. UU. (Stripe Atlas) con sus declaraciones. Se revisa cuando haya una empresa en
un pais de Stripe y facturacion suficiente para pagar un asesor (orientativo: mas de 20.000 USD al
mes).

**Lemon Squeezy.** Mismo precio de partida que Paddle, con recargos internacionales y de
renovacion. Desde 2024 es de Stripe y su equipo construye Stripe Managed Payments y prepara la
migracion: empezar aqui es planear una migracion.

**Stripe Managed Payments.** El MoR de Stripe cobra un 3,5 % ademas de las comisiones de Stripe y
solo admite empresas de una lista corta de paises. Con el pais de facturacion sin confirmar, puede
no estar disponible.

**Polar como primera opcion.** Alta rapida y una API comoda, pero recarga un 1,5 % con tarjetas no
estadounidenses, que serian buena parte de las ventas, es una empresa joven y depende de Stripe
Connect. Queda como plan B si Paddle tarda en aprobar o rechaza la cuenta.

**FastSpring.** MoR orientado a empresas con volumen, con una comision negociada mas alta para
empezar.
