#!/usr/bin/env bash
# Valida las recetas comunitarias contra su manifest. Se puede llamar desde
# cualquier directorio; las rutas se resuelven respecto a la raíz del repo.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$root/community-recipes/manifest.tsv"

trim() {
    local s="$1"
    s="${s#"${s%%[![:space:]]*}"}"
    s="${s%"${s##*[![:space:]]}"}"
    printf '%s' "$s"
}

if [[ ! -f "$manifest" ]]; then
    echo "FALLO manifest: no existe $manifest" >&2
    exit 1
fi

checked=0
failures=0

while IFS=$'\t' read -r id app match_any winetricks fixture || [[ -n "${id:-}" ]]; do
    id="$(trim "${id:-}")"
    if [[ -z "$id" || "$id" == "id" || "$id" == \#* ]]; then
        continue
    fi
    match_any="$(trim "${match_any:-}")"
    winetricks="$(trim "${winetricks:-}")"
    fixture="$(trim "${fixture:-}")"
    fixture="${fixture%$'\r'}"
    checked=$((checked + 1))

    reasons=()
    app_dir="$root/community-recipes/apps/$id"
    recipe="$app_dir/recipe.recipe"
    log="$app_dir/$fixture"

    if [[ ! -f "$recipe" ]]; then
        reasons+=("falta community-recipes/apps/$id/recipe.recipe")
    fi
    if [[ -z "$fixture" ]]; then
        reasons+=("fixture vacío en el manifest")
    elif [[ ! -f "$log" ]]; then
        reasons+=("falta el log $fixture")
    fi

    if [[ -f "$recipe" && -f "$log" ]]; then
        IFS=',' read -r -a needles <<< "$match_any"
        for needle in "${needles[@]}"; do
            needle="$(trim "$needle")"
            if [[ -z "$needle" ]]; then
                continue
            fi
            if ! grep -Fqi -- "$needle" "$log"; then
                reasons+=("el log no contiene '$needle'")
            fi
        done

        declared=""
        while IFS= read -r line || [[ -n "$line" ]]; do
            line="$(trim "$line")"
            if [[ "$line" == action_winetricks* ]]; then
                declared="$(trim "${line#*=}")"
                break
            fi
        done < "$recipe"
        # "-" significa "sin winetricks": recetas de diagnóstico con log real.
        if [[ "$winetricks" != "-" && "${declared,,}" != "${winetricks,,}" ]]; then
            reasons+=("action_winetricks esperado '$winetricks', encontrado '$declared'")
        fi
    fi

    if (( ${#reasons[@]} == 0 )); then
        echo "OK $id"
    else
        echo "FALLO $id: $(IFS='; '; echo "${reasons[*]}")"
        failures=$((failures + 1))
    fi
done < "$manifest"

echo "Recetas validadas: $checked; fallos: $failures"
if (( failures > 0 )); then
    exit 1
fi
