import { type Container, createContainer } from './composition-root';

/**
 * El contenedor de esta carga de la pagina. Un modulo ES se evalua una sola vez,
 * asi que cada carga tiene uno y lo comparten root.tsx (que lo entrega a la
 * interfaz) y los loaders de las rutas.
 */
export const container: Container = createContainer(import.meta.env);
