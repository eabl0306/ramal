// IBM Plex empaquetada con la app, nunca desde Google Fonts: ni la CSP de Tauri
// ni el uso sin conexion lo permitirian (architecture.md 6.8). Escritorio y web
// importan este modulo una vez en su entrada; VS Code no, usa la fuente del editor.
import '@fontsource/ibm-plex-sans/400.css';
import '@fontsource/ibm-plex-sans/500.css';
import '@fontsource/ibm-plex-sans/600.css';
import '@fontsource/ibm-plex-sans/700.css';
import '@fontsource/ibm-plex-mono/400.css';
import '@fontsource/ibm-plex-mono/500.css';
import '@fontsource/ibm-plex-mono/600.css';
