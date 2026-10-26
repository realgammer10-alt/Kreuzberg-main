```php title="PHP (ReactPHP)"
<?php

declare(strict_types=1);

// Requires: composer require react/promise ^3.0 react/event-loop ^1.0
use Kreuzberg\Kreuzberg;
use Kreuzberg\Async\ReactBridge;

$kreuzberg = new Kreuzberg();

// Single file extraction with ReactPHP Promise
$deferred = $kreuzberg->extractFileAsync('document.pdf');
$promise = ReactBridge::toPromise($deferred);

$promise->then(
    function ($result) {
        echo "Content: {$result->content}\n";
        echo "MIME type: {$result->mimeType}\n";
    },
    function (\Throwable $error) {
        echo "Extraction failed: {$error->getMessage()}\n";
    }
);
```
