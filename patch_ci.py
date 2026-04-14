import re

with open(".github/workflows/ci.yml", "r") as f:
    content = f.read()

# Add step to upload FV coverage report
insertion = """
      - name: Generate FV Coverage Report
        run: ./tools/generate_fv_coverage.sh

      - name: Upload FV Coverage Report
        uses: actions/upload-artifact@v4
        with:
          name: fv-coverage-report
          path: target/fv_coverage/index.html
"""

content = content.replace("      - name: Generate FV Coverage Report\n        run: ./tools/generate_fv_coverage.sh", insertion)

with open(".github/workflows/ci.yml", "w") as f:
    f.write(content)
