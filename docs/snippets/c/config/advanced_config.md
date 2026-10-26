```c title="C"
#include "kreuzberg.h"
#include <stdio.h>

int main(void) {
    struct ConfigBuilder *builder = kreuzberg_config_builder_new();
    kreuzberg_config_builder_set_use_cache(builder, 1);
    kreuzberg_config_builder_set_include_document_structure(builder, 1);
    kreuzberg_config_builder_set_ocr(builder,
        "{\"tesseract\":{\"language\":\"eng\"}}");

    ExtractionConfig *config = kreuzberg_config_builder_build(builder);

    struct CExtractionResult *result =
        kreuzberg_extract_file_sync_with_config("scan.pdf",
            kreuzberg_config_to_json(config));
    if (result && result->success) {
        printf("%s\n", result->content);
    }

    kreuzberg_free_result(result);
    kreuzberg_config_free(config);
    return 0;
}
```
