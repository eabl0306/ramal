import { Buffer } from 'node:buffer';

import { describe, expect, it } from 'vitest';

import {
  FrameDecoder,
  FrameError,
  MAX_FRAME_BYTES,
  MAX_HEADER_BYTES,
  encodeFrame,
  parseContentLength,
} from './frame-codec';

describe('marcos Content-Length', () => {
  it('codifica la longitud en bytes de UTF-8, no en caracteres', () => {
    const frame = encodeFrame('{"a":"ñ"}');
    expect(frame.toString('utf8')).toBe('Content-Length: 10\r\n\r\n{"a":"ñ"}');
  });

  it('decodifica un marco completo', () => {
    const decoder = new FrameDecoder();
    expect(decoder.push(encodeFrame('{"id":1}'))).toEqual(['{"id":1}']);
  });

  it('junta un marco que llega en trozos, tambien partiendo un caracter multibyte', () => {
    const decoder = new FrameDecoder();
    const frame = encodeFrame('{"summary":"añadir grafo"}');
    const bodies = [...frame].map((byte) => decoder.push(Buffer.from([byte]))).flat();
    expect(bodies).toEqual(['{"summary":"añadir grafo"}']);
  });

  it('saca varios marcos de un mismo trozo y guarda el resto para despues', () => {
    const decoder = new FrameDecoder();
    const both = Buffer.concat([encodeFrame('{"id":1}'), encodeFrame('{"id":2}')]);
    const third = encodeFrame('{"id":3}');
    expect(decoder.push(Buffer.concat([both, third.subarray(0, 5)]))).toEqual([
      '{"id":1}',
      '{"id":2}',
    ]);
    expect(decoder.push(third.subarray(5))).toEqual(['{"id":3}']);
  });

  it('ignora otras cabeceras y no distingue mayusculas en el nombre', () => {
    const decoder = new FrameDecoder();
    const raw = 'Content-Type: application/json\r\ncontent-length: 2\r\n\r\n{}';
    expect(decoder.push(Buffer.from(raw))).toEqual(['{}']);
  });

  it('falla sin Content-Length o con un valor que no es un numero', () => {
    expect(() => parseContentLength('Content-Type: x')).toThrow(FrameError);
    expect(() => parseContentLength('Content-Length: 12a')).toThrow('invalid Content-Length');
    expect(() => parseContentLength('Content-Length: -1')).toThrow(FrameError);
  });

  it('rechaza un marco de mas de 32 MiB antes de esperar su cuerpo', () => {
    const decoder = new FrameDecoder();
    const header = `Content-Length: ${String(MAX_FRAME_BYTES + 1)}\r\n\r\n`;
    expect(() => decoder.push(Buffer.from(header))).toThrow('exceeds the limit');
  });

  it('rechaza unas cabeceras que no terminan nunca', () => {
    const decoder = new FrameDecoder();
    expect(() => decoder.push(Buffer.alloc(MAX_HEADER_BYTES + 1, 'a'))).toThrow('too long');
  });

  it('espera mientras las cabeceras o el cuerpo estan a medias', () => {
    const decoder = new FrameDecoder();
    expect(decoder.push(Buffer.from('Content-Len'))).toEqual([]);
    expect(decoder.push(Buffer.from('gth: 4\r\n\r\n{}'))).toEqual([]);
    expect(decoder.push(Buffer.from('[]'))).toEqual(['{}[]']);
    expect(decoder.push(Buffer.alloc(0))).toEqual([]);
  });
});
