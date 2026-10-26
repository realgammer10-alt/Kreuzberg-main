```php title="PHP (Amp v3+)"
<?php

declare(strict_types=1);

// Requires: composer require amphp/amp ^3.0
use Kreuzberg\Kreuzberg;
use Kreuzberg\Async\AmpBridge;

$kreuzberg = new Kreuzberg();

// Single file extraction with Amp Future
$deferred = $kreuzberg->extractFileAsync('document.pdf');
$future = AmpBridge::toFuture($deferred);
$result = $future->await();
echo $result->content;

// Batch extraction with Amp Future
$files = ['doc1.pdf', 'doc2.docx', 'doc3.xlsx'];
$batchDeferred = $kreuzberg->batchExtractFilesAsync($files);
$batchFuture = AmpBridge::toBatchFuture($batchDeferred);
$results = $batchFuture->await();

foreach ($results as $i => $result) {
    echo "{$files[$i]}: {$result->content}\n";
}
```
