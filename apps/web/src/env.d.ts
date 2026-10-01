// Variables de entorno que la web lee al construir (import.meta.env). Solo las
// VITE_* llegan al navegador; ninguna es secreta.
interface ImportMetaEnv {
  /** Origen de la API. Vacia o ausente es el mismo origen, el caso normal. */
  readonly VITE_API_URL?: string;
}
