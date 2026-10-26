```c title="C"
#include "kreuzberg.h"
#include <stdio.h>

int main(void) {
    struct CExtractionResult *result = kreuzberg_extract_file_sync("spreadsheet.xlsx");
    if (!result || !result->success) {
        fprintf(stderr, "Error: %s\n", kreuzberg_get_error_details().message);
        return 1;
    }

    if (result->tables_json) {
        printf("Tables (JSON): %s\n", result->tables_json);
    } else {
        printf("No tables found\n");
    }

    kreuzberg_free_result(result);
    return 0;
}
```
