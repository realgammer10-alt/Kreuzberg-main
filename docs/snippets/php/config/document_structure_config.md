```php title="Document Structure Config (PHP)"
<?php
use Kreuzberg\ExtractionConfig;
use Kreuzberg\Kreuzberg;

$config = new ExtractionConfig(includeDocumentStructure: true);

$result = Kreuzberg::extractFileSync('document.pdf', $config);

if ($result->document !== null) {
    foreach ($result->document->nodes as $node) {
        echo "[{$node->content->nodeType}]\n";
    }
}
?>
```
