#!/usr/bin/env bash
# Crea el esqueleto de una receta comunitaria. El log queda VACÍO (solo
# comentarios): nunca se generan líneas de log falsas.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
    cat <<'EOF'
Uso: tools/new-recipe.sh <id>

Crea community-recipes/apps/<id>/recipe.recipe y
community-recipes/apps/<id>/logs/<id>.log (solo comentarios).

El id debe empezar por 'community-' y contener solo [a-z0-9-].
Este script NO modifica community-recipes/manifest.tsv: imprime la fila que
debes añadir a mano. Consulta community-recipes/REAL-LOGS.md.
EOF
}

case "${1:-}" in
    --help | -h)
        usage
        exit 0
        ;;
esac

if (( $# != 1 )); then
    echo "ERROR: se espera exactamente un id" >&2
    usage >&2
    exit 2
fi

id="$1"
if [[ ! "$id" =~ ^community-[a-z0-9-]+$ ]]; then
    echo "ERROR: id no válido '$id': debe empezar por community- y usar solo [a-z0-9-]" >&2
    exit 1
fi

dir="$root/community-recipes/apps/$id"
if [[ -e "$dir" ]]; then
    echo "ERROR: ya existe $dir; no se sobrescribe nada" >&2
    exit 1
fi

mkdir -p "$dir/logs"

cat > "$dir/recipe.recipe" <<'EOF'
# Receta comunitaria (plantilla). Sustituye los placeholders antes de validar.
# Copia las cadenas de match_log_any literalmente de tu log REAL redactado.
# Consulta community-recipes/REAL-LOGS.md.
[ID_AQUI]
confidence = 0.80
priority = 10
match_log_any = CADENA_DEL_LOG_REAL_AQUI
# match_detector = nsis
# match_exit_code = 1
# match_file_missing = drive_c/windows/system32/ejemplo.dll
action_winetricks = VERB_DE_WINETRICKS_AQUI
# action_override = oleaut32=native
# action_env = WINEARCH=win64
# action_engine = wine-ge
# action_reset_prefix = true
action_message = Aplicando receta comunitaria...
action_retry = true
EOF

# Sustituimos el id solo en la línea del DSL (el resto del archivo es plantilla).
sed -i "s/^\[ID_AQUI\]$/[$id]/" "$dir/recipe.recipe"

cat > "$dir/logs/$id.log" <<'EOF'
# LOG REAL REDACTADO: pega aquí SOLO el fragmento de una instalación real.
#
# Este archivo contiene únicamente comentarios a propósito. No hay líneas de
# log inventadas: no copies los fixtures anonimizados de otras recetas.
#
# 1) Ejecuta la instalación y localiza el fragmento útil del log de Wine.
# 2) Redacta: tools/redact-log.sh log-original.log > logs/ID.log
# 3) Pega aquí el resultado (err:module:import_dll, err:seh, fixme:msi,
#    mensajes MSI/NSIS/Inno y el código de salida).
# 4) Conserva al menos una cadena literal declarada en match_log_any.
# 5) Confirma consentimiento para publicar: community-recipes/REAL-LOGS.md.
EOF

sed -i "s@logs/ID\.log@logs/$id.log@" "$dir/logs/$id.log"

echo "Creado:"
echo "  community-recipes/apps/$id/recipe.recipe"
echo "  community-recipes/apps/$id/logs/$id.log"
echo
echo "Pasos siguientes (NO se ha tocado manifest.tsv):"
echo "  1. Sustituye los placeholders de recipe.recipe con datos de tu log real."
echo "  2. Pega el log REAL redactado en logs/$id.log (tools/redact-log.sh)."
echo "  3. Añade esta fila (TABs) a community-recipes/manifest.tsv:"
printf '     %s\t%s\t%s\t%s\t%s\n' \
    "$id" "<app>" "<cadenas de match_log_any>" "<verb de winetricks>" "logs/$id.log"
echo "  4. tools/validate-recipes.sh"
echo "  5. crisol recipes import community-recipes/apps"
