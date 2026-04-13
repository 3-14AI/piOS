#!/bin/bash
set -e
echo "Running Hardware Stress Test via QEMU..."

SUCCESS_COUNT=0
ITERATIONS=3

for ((i=1; i<=ITERATIONS; i++)); do
  echo "Stress iteration $i..."

  # run_qemu_test.sh inherently sets -e and will fail the script if it crashes.
  ./tools/run_qemu_test.sh
  SUCCESS_COUNT=$((SUCCESS_COUNT+1))
done

if [ "$SUCCESS_COUNT" -eq "$ITERATIONS" ]; then
    echo "Hardware Stress Test Passed: $SUCCESS_COUNT/$ITERATIONS"
else
    echo "Hardware Stress Test Failed: $SUCCESS_COUNT/$ITERATIONS"
    # return an error
    false
fi
