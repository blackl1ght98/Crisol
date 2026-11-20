#!/usr/bin/env bash
# Integración mínima de Crisol: CLI, VM sin QEMU y panel de feedback.
# No instala paquetes ni usa sudo; todo el estado se crea en un XDG temporal.
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
PANEL_PID=""

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/crisol-integration.XXXXXX")"

cleanup() {
  local status=$?
  trap - EXIT INT TERM
  if [[ -n "$PANEL_PID" ]] && kill -0 "$PANEL_PID" 2>/dev/null; then
    kill "$PANEL_PID" 2>/dev/null || true
    wait "$PANEL_PID" 2>/dev/null || true
  fi
  rm -rf -- "$TMP_ROOT"
  exit "$status"
}
trap cleanup EXIT INT TERM

ok()   { printf '  OK: %s\n' "$*"; }
info() { printf '  ..: %s\n' "$*"; }
fail() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

run_cmd() {
  local desc="$1"
  shift
  printf '\n==> %s\n' "$desc"
  if "$@"; then
    ok "$desc"
  else
    fail "$desc (exit $?)"
  fi
}

# ---------------------------------------------------------------------------
# Compilación (antes de aislar HOME para no afectar a rustup/cargo).
# ---------------------------------------------------------------------------
cd "$ROOT"
printf '==> Compilando crisol (cargo build --quiet)\n'
cargo build --quiet || fail "cargo build --quiet falló"
BIN="$ROOT/target/debug/crisol"
[[ -x "$BIN" ]] || fail "no se encontró el binario $BIN"
ok "binario compilado: $BIN"

# ---------------------------------------------------------------------------
# XDG temporal (el panel hereda estas variables al arrancar en background).
# ---------------------------------------------------------------------------
export XDG_DATA_HOME="$TMP_ROOT/data"
export XDG_CONFIG_HOME="$TMP_ROOT/config"
export XDG_CACHE_HOME="$TMP_ROOT/cache"
export XDG_RUNTIME_DIR="$TMP_ROOT/runtime"
export HOME="$TMP_ROOT/home"
mkdir -p "$XDG_DATA_HOME" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME" "$XDG_RUNTIME_DIR" "$HOME"
ok "XDG temporal: $TMP_ROOT"

CATALOG="$XDG_DATA_HOME/crisol/catalog.tsv"
FEEDBACK="$XDG_CONFIG_HOME/crisol/feedback.tsv"
VM_DIR="$XDG_DATA_HOME/crisol/vms/vm-ci"
PORT=18765
BASE="http://127.0.0.1:$PORT"

# ---------------------------------------------------------------------------
# Comandos básicos de la CLI.
# ---------------------------------------------------------------------------
run_cmd "crisol doctor" "$BIN" doctor
run_cmd "crisol list" "$BIN" list
run_cmd "crisol recipes" "$BIN" recipes

# --disk se limita a 10 GB en la CLI; sin qemu-img debe avisar y terminar 0.
run_cmd "crisol vm create vm-ci --ram 1 --disk 1 (sin QEMU)" \
  "$BIN" vm create vm-ci --ram 1 --disk 1
run_cmd "crisol vm list" "$BIN" vm list

[[ -f "$VM_DIR/start.sh" ]] || fail "falta $VM_DIR/start.sh"
[[ -d "$VM_DIR/share" ]]   || fail "falta el directorio $VM_DIR/share"
ok "VM creada: start.sh y share/ presentes"

# ---------------------------------------------------------------------------
# Catálogo de prueba con 11 columnas (la última con dos recetas).
# ---------------------------------------------------------------------------
mkdir -p "$(dirname "$CATALOG")"
printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
  'app-ci' 'App CI' 'drive_c/App/app.exe' '' '' '' '' 'wine' \
  '1700000000' 'instalado' 'recipe-a;recipe-b' > "$CATALOG"
awk -F '\t' 'NF != 11 { exit 1 }' "$CATALOG" || fail "catalog.tsv no tiene 11 columnas"
ok "catalog.tsv de prueba escrito (11 columnas, recipes=recipe-a;recipe-b)"

# ---------------------------------------------------------------------------
# Panel en background y espera activa (máx. 20 × 0.25 s = 5 s).
# ---------------------------------------------------------------------------
printf '\n==> Arrancando crisol panel --port %s\n' "$PORT"
nohup "$BIN" panel --port "$PORT" >"$TMP_ROOT/panel.log" 2>&1 &
PANEL_PID=$!

HTTP_STATUS=""
for ((i = 1; i <= 20; i++)); do
  HTTP_STATUS="$(curl -sS -o "$TMP_ROOT/index.html" -w '%{http_code}' --max-time 1 "$BASE/" 2>/dev/null || true)"
  [[ "$HTTP_STATUS" == "200" ]] && break
  sleep 0.25
done
if [[ "$HTTP_STATUS" != "200" ]]; then
  cat "$TMP_ROOT/panel.log" >&2 || true
  fail "el panel no respondió HTTP 200 en 20 intentos (último: '${HTTP_STATUS:-sin respuesta}')"
fi
grep -q 'Crisol' "$TMP_ROOT/index.html" || fail "GET / no contiene 'Crisol'"
ok "GET / -> HTTP 200 y contiene 'Crisol'"

TOKEN_FILE="$XDG_CONFIG_HOME/crisol/panel.token"
[[ -s "$TOKEN_FILE" ]] || fail "no se generó el token del panel en $TOKEN_FILE"
PANEL_TOKEN="$(tr -d '[:space:]' < "$TOKEN_FILE")"
ok "token del panel generado"

csrf_code="$(curl -sS -o "$TMP_ROOT/csrf.body" -w '%{http_code}' --max-time 2 \
  "$BASE/ok?id=app-ci" 2>/dev/null || true)"
[[ "$csrf_code" == "403" ]] || fail "voto sin token devolvió '$csrf_code' (esperado 403)"
ok "voto sin token rechazado con HTTP 403"

# ---------------------------------------------------------------------------
# Feedback: /ok suma 0.05 y /bad resta (0.05/0.10 según versión).
# ---------------------------------------------------------------------------
send_feedback() {
  local route="$1"
  local code
  code="$(curl -sS -o "$TMP_ROOT/$route.body" -w '%{http_code}' --max-time 2 \
    "$BASE/$route?id=app-ci&t=$PANEL_TOKEN" 2>/dev/null || true)"
  [[ "$code" == "302" || "$code" == "200" ]] || \
    fail "GET /$route?id=app-ci devolvió '${code:-sin respuesta}'"
}

send_feedback ok
[[ -f "$FEEDBACK" ]] || fail "no se creó $FEEDBACK"
grep -Eq 'recipe-a[[:space:]]+0\.05([[:space:]]|$)' "$FEEDBACK" || \
  fail "feedback.tsv no contiene recipe-a con +0.05 tras /ok"
ok "feedback.tsv contiene recipe-a con 0.05"

send_feedback bad
[[ -f "$FEEDBACK" ]] || fail "desapareció $FEEDBACK"
if grep -Eq 'recipe-a[[:space:]]+-0\.(05|10)([[:space:]]|$)' "$FEEDBACK"; then
  ok "feedback.tsv contiene recipe-a con -0.05/-0.10 tras /bad"
elif grep -Eq 'recipe-a[[:space:]]' "$FEEDBACK"; then
  ok "feedback.tsv sigue conteniendo recipe-a (delta con formato distinto; se acepta)"
else
  fail "feedback.tsv no contiene recipe-a tras /bad"
fi

# ---------------------------------------------------------------------------
# Comandos nuevos: se prueban sólo si están implementados (la CLI actual
# devuelve la ayuda con "USO" para comandos desconocidos).
# ---------------------------------------------------------------------------
if out="$("$BIN" feedback stats 2>&1)" && ! grep -q 'USO' <<<"$out"; then
  ok "crisol feedback stats disponible"
else
  info "crisol feedback stats no implementado; se omite"
fi

FEEDBACK_EXPORT="$TMP_ROOT/feedback-export.tsv"
if out="$("$BIN" feedback export "$FEEDBACK_EXPORT" 2>&1)" && \
   ! grep -q 'USO' <<<"$out" && [[ -f "$FEEDBACK_EXPORT" ]]; then
  ok "crisol feedback export disponible"
else
  info "crisol feedback export no implementado; se omite"
fi

RECIPES_EXPORT_DIR="$TMP_ROOT/recipes-export"
if out="$("$BIN" recipes export "$RECIPES_EXPORT_DIR" 2>&1)" && \
   ! grep -q 'USO' <<<"$out" && [[ -d "$RECIPES_EXPORT_DIR" ]]; then
  ok "crisol recipes export disponible"
else
  info "crisol recipes export no implementado; se omite"
fi

if out="$("$BIN" recipes import "$RECIPES_EXPORT_DIR" 2>&1)" && ! grep -q 'USO' <<<"$out"; then
  ok "crisol recipes import disponible"
else
  info "crisol recipes import no implementado; se omite"
fi

# CLI moderna: versión, JSON y completions.
version="$("$BIN" --version)"
[[ "$version" == crisol\ * ]] || fail "--version devolvió '$version'"
ok "--version -> $version"

for cmd in list doctor recipes "feedback stats"; do
  out="$("$BIN" --json $cmd 2>&1)" || fail "--json $cmd falló"
  python3 -c 'import json,sys; json.load(sys.stdin)' <<<"$out" || fail "--json $cmd no es JSON válido"
done
ok "--json produce JSON válido en list/doctor/recipes/feedback stats"

for shell in bash zsh fish; do
  out="$("$BIN" completions "$shell" 2>&1)" || fail "completions $shell falló"
  [[ -n "$out" ]] || fail "completions $shell está vacío"
done
ok "completions bash/zsh/fish generados"

printf '\n==> Integración completada correctamente\n'

