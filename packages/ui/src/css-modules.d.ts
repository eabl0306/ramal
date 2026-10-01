// Tipos de los CSS Modules y de los CSS importados por su efecto (temas, fuentes,
// codicons). Vite los resuelve en los tres hosts; TypeScript solo necesita saber
// que existen.
declare module '*.module.css' {
  const classes: Readonly<Record<string, string>>;
  export default classes;
}

declare module '*.css';
