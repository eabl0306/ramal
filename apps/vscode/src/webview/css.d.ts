// Tipos de los CSS Modules y de los CSS importados por su efecto (el tema de
// @ramal/ui). Vite los resuelve; TypeScript solo necesita saber que existen.
declare module '*.module.css' {
  const classes: Readonly<Record<string, string>>;
  export default classes;
}

declare module '*.css';
