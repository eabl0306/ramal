import { noJsxLiteralText } from './no-jsx-literal-text.js';
import { typescriptRuleTester } from './rule-tester.js';

const filename = 'src/components/panel.tsx';

function valid(code, options) {
  return options === undefined ? { code, filename } : { code, filename, options };
}

function invalid(code, messageIds, options) {
  const errors = messageIds.map((messageId) => ({ messageId }));
  return options === undefined ? { code, filename, errors } : { code, filename, errors, options };
}

typescriptRuleTester().run('no-jsx-literal-text', noJsxLiteralText, {
  valid: [
    valid('const a = <p>{t("panel.title")}</p>;'),
    valid('const a = <img alt="" src={src} />;'),
    valid('const a = <span> · </span>;'),
    valid('const a = <p>{count}</p>;'),
    valid('const a = <p>{42} / {total}</p>;'),
    valid('const a = <input placeholder={t("search.placeholder")} />;'),
    valid('const a = <div className="graph" data-testid="commit-graph" />;'),
    valid('const a = <span>{"/"}</span>;'),
    valid('const a = <p>{`${count}`}</p>;'),
    valid('const a = <Foo title="Hola" />;', [{ attributes: ['tooltip'] }]),
  ],
  invalid: [
    invalid('const a = <p>Hola</p>;', ['literalText']),
    invalid('const a = <p>Ñandú</p>;', ['literalText']),
    invalid('const a = <>Texto suelto</>;', ['literalText']),
    invalid('const a = <p>{"Guardar"}</p>;', ['literalText']),
    invalid('const a = <p>{`Guardar`}</p>;', ['literalText']),
    invalid('const a = <p>{ok ? "Si" : "No"}</p>;', ['literalText', 'literalText']),
    invalid('const a = <p>{name || "Anonimo"}</p>;', ['literalText']),
    invalid('const a = <input placeholder="Buscar" />;', ['literalAttribute']),
    invalid('const a = <img alt="Logo de Git Rudder" src={src} />;', ['literalAttribute']),
    invalid('const a = <button aria-label="Cerrar" />;', ['literalAttribute']),
    invalid('const a = <abbr title={"Titulo"} />;', ['literalAttribute']),
    invalid('const a = <Field label={`Nombre`} />;', ['literalAttribute']),
    invalid(
      'const a = <Foo tooltip="Hola" />;',
      ['literalAttribute'],
      [{ attributes: ['tooltip'] }],
    ),
  ],
});
