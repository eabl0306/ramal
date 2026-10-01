// Tipos de los CSS Modules. Vite los resuelve en escritorio y en los webviews de
// VS Code; TypeScript solo necesita saber que existen.
declare module '*.module.css' {
  const classes: Readonly<Record<string, string>>;
  export default classes;
}
