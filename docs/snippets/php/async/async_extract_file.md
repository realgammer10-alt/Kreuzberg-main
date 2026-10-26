```php title="PHP"
<?php

declare(strict_types=1);

use Kreuzberg\Kreuzberg;
use Kreuzberg\Exceptions\KreuzbergException;
use function Kreuzberg\extract_file_async;

// OOP API: async file extraction
$kreuzberg = new Kreuzberg();
$deferred = $kreuzberg->extractFileAsync('document.pdf');

// Non-blocking: check if ready
if ($deferred->isReady()) {
    $result = $deferred->getResult();
    echo $result->content;
}

// Non-blocking: try to get result (returns null if pending)
$result = $deferred->tryGetResult();
if ($result !== null) {
    echo $result->content;
}

// Blocking: wait until ready
$result = $deferred->getResult();
echo $result->content;

// Blocking with timeout (5 seconds)
$result = $deferred->wait(5000);
if ($result !== null) {
    echo $result->content;
} else {
    echo "Extraction timed out\n";
}

// Procedural API
$deferred = extract_file_async('document.pdf');
$result = $deferred->getResult();
echo $result->content;

// Static API
$deferred = Kreuzberg::extractFileAsyncStatic('document.pdf');
$result = $deferred->getResult();
echo $result->content;
```
