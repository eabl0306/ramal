-- Esquema base de Ramal (fase F0).
--
-- Todavia no hay dominio que guardar, asi que esta migracion solo prepara el
-- terreno, sin tablas inventadas:
--
--   - citext, para que correos y nombres de usuario no distingan mayusculas sin
--     acordarse de un lower() en cada consulta (lo usan las tablas de F1).
--   - schema_metadata, una fila unica con la version de esquema aplicada. Los
--     tests de integracion la usan para comprobar que las migraciones se
--     aplicaron y que son idempotentes.
--   - touch_updated_at, el disparador que escribe updated_at en cada tabla que
--     lo tenga: un disparador no se olvida en la sexta sentencia UPDATE.
--
-- El modelo completo, fase por fase, esta en docs/data-model.md.

CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE IF NOT EXISTS schema_metadata (
    singleton      boolean     PRIMARY KEY DEFAULT TRUE,
    schema_version text        NOT NULL,
    updated_at     timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT schema_metadata_singleton CHECK (singleton)
);

COMMENT ON TABLE schema_metadata IS
    'Una sola fila con la version de esquema aplicada. Sin datos de negocio.';

CREATE OR REPLACE FUNCTION touch_updated_at() RETURNS trigger AS $$
BEGIN
    NEW.updated_at := now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER schema_metadata_touch BEFORE UPDATE ON schema_metadata
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

INSERT INTO schema_metadata (schema_version)
VALUES ('0.1.0')
ON CONFLICT (singleton) DO UPDATE
    SET schema_version = EXCLUDED.schema_version;
