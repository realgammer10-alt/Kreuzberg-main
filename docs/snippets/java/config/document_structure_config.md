```java title="Document Structure Config (Java)"
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionConfig;
import dev.kreuzberg.ExtractionResult;

ExtractionConfig config = ExtractionConfig.builder()
    .includeDocumentStructure(true)
    .build();

ExtractionResult result = Kreuzberg.extractFileSync("document.pdf", config);

if (result.getDocumentStructure().isPresent()) {
    var document = result.getDocumentStructure().get();
    for (var node : document.nodes()) {
        System.out.println("[" + node.content().nodeType() + "]");
    }
}
```
