```r title="R"
library(kreuzberg)

# Extract a PDF file
result <- extract_file_sync("example.pdf")

# Print a preview of the extracted content
content_preview <- substr(content(result), 1L, 200L)
cat("Content preview:\n")
cat(content_preview)
cat("\n...\n")
```
