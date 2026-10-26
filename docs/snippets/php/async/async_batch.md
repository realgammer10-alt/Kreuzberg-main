```php title="PHP"
<?php

declare(strict_types=1);

use Kreuzberg\Kreuzberg;
use function Kreuzberg\batch_extract_files_async;

$kreuzberg = new Kreuzberg();

// Async batch file extraction
$files = ['doc1.pdf', 'doc2.docx', 'doc3.xlsx'];
$deferred = $kreuzberg->batchExtractFilesAsync($files);

// Do other work while extraction runs...
processOtherTasks();

// Block until all results are ready
$results = $deferred->getResults();

foreach ($results as $i => $result) {
    echo "{$files[$i]}: " . strlen($result->content) . " chars\n";
}

// With timeout
$deferred = $kreuzberg->batchExtractFilesAsync($files);
$results = $deferred->waitBatch(10000); // 10 second timeout

if ($results !== null) {
    foreach ($results as $result) {
        echo $result->content . "\n";
    }
} else {
    echo "Batch extraction timed out\n";
}

// Procedural API
$deferred = batch_extract_files_async($files);
$results = $deferred->getResults();
```
