for BIN in $(cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].targets[] | select(.kind[]=="bin") | .name'); do
    FLASH=$(cargo size --release --bin "$BIN" -- -A | awk '/\.text|\.rodata|\.data/ {sum += strtonum($2)} END {print sum}')
    RAM=$(cargo size --release --bin "$BIN" -- -A | awk '/\.data|\.bss|\.uninit/ {sum += strtonum($2)} END {print sum}')

    echo "$BIN: flash=$FLASH bytes, ram(static)=$RAM bytes"
done
