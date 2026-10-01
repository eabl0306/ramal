#!/usr/bin/env node
// vsce exige un fichero de licencia en la raiz de la extension y la VSIX tiene
// que llevar la del proyecto. La fuente es LICENSE.md de la raiz del monorepo;
// aqui solo se copia justo antes de empaquetar (la copia no se versiona).
import { copyFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const source = fileURLToPath(new URL('../../../LICENSE.md', import.meta.url));
const target = fileURLToPath(new URL('../LICENSE.md', import.meta.url));

copyFileSync(source, target);
