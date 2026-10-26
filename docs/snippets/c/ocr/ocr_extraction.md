```c title="C"
#include "kreuzberg.h"
#include <stdio.h>

int main(void) {
    struct ConfigBuilder *builder = kreuzberg_config_builder_new();
    kreuzberg_config_builder_set_ocr(builder,
        "{\"tesseract\":{\"language\":\"eng\"}}");
    ExtractionConfig *config = kreuzberg_config_builder_build(builder);

    char *config_json = kreuzberg_config_to_json(config);
    struct CExtractionResult *result =
        kreuzberg_extract_file_sync_with_config("scanned.png", config_json);

    if (result && result->success) {
        printf("OCR text: %s\n", result->content);
    } else {
        fprintf(stderr, "OCR error: %s\n", kreuzberg_get_error_details().message);
    }

    kreuzberg_free_result(result);
    kreuzberg_free_string(config_json);
    kreuzberg_config_free(config);
    return 0;
}
```
