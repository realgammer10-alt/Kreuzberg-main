```c title="C"
#include "kreuzberg.h"
#include <stdio.h>

int main(void) {
    struct CExtractionResult *result = kreuzberg_extract_file_sync("document.pdf");
    if (!result || !result->success) {
        fprintf(stderr, "Error: %s\n", kreuzberg_get_error_details().message);
        return 1;
    }

    printf("Content: %s\n", result->content);
    printf("MIME: %s\n", result->mime_type);

    if (result->language)
        printf("Language: %s\n", result->language);
    if (result->date)
        printf("Date: %s\n", result->date);
    if (result->subject)
        printf("Subject: %s\n", result->subject);
    if (result->metadata_json)
        printf("Metadata: %s\n", result->metadata_json);

    kreuzberg_free_result(result);
    return 0;
}
```
