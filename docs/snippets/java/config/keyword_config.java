import com.kreuzberg.Kreuzberg;
import com.kreuzberg.config.ExtractionConfig;
import com.kreuzberg.config.KeywordConfig;
import com.kreuzberg.keywords.YakeParams;
import com.kreuzberg.keywords.RakeParams;
import com.kreuzberg.result.ExtractionResult;

// Example 1: Basic YAKE configuration
// Uses YAKE algorithm with default parameters and English stopword filtering
public class KeywordConfigExample {

    public static void basicYake() throws Exception {
        ExtractionConfig config = new ExtractionConfig.Builder()
            .keywords(new KeywordConfig.Builder()
                .algorithm("yake")
                .maxKeywords(10)
                .minScore(0.0f)
                .ngramRange(1, 3)
                .language("en")
                .yakeParams(null)
                .rakeParams(null)
                .build())
            .build();

        ExtractionResult result = Kreuzberg.extractFile("document.pdf", config);
        System.out.println("Keywords: " + result.getKeywords());
    }

    // Example 2: Advanced YAKE with custom parameters
    // Fine-tunes YAKE with custom window size for co-occurrence analysis
    public static void advancedYake() throws Exception {
        ExtractionConfig config = new ExtractionConfig.Builder()
            .keywords(new KeywordConfig.Builder()
                .algorithm("yake")
                .maxKeywords(15)
                .minScore(0.1f)
                .ngramRange(1, 2)
                .language("en")
                .yakeParams(new YakeParams.Builder()
                    .windowSize(1)
                    .build())
                .rakeParams(null)
                .build())
            .build();

        ExtractionResult result = Kreuzberg.extractFile("document.pdf", config);
        System.out.println("Keywords: " + result.getKeywords());
    }

    // Example 3: RAKE configuration
    // Uses RAKE algorithm for rapid keyword extraction with phrase constraints
    public static void rakeConfig() throws Exception {
        ExtractionConfig config = new ExtractionConfig.Builder()
            .keywords(new KeywordConfig.Builder()
                .algorithm("rake")
                .maxKeywords(10)
                .minScore(5.0f)
                .ngramRange(1, 3)
                .language("en")
                .yakeParams(null)
                .rakeParams(new RakeParams.Builder()
                    .minWordLength(1)
                    .maxWordsPerPhrase(3)
                    .build())
                .build())
            .build();

        ExtractionResult result = Kreuzberg.extractFile("document.pdf", config);
        System.out.println("Keywords: " + result.getKeywords());
    }

    public static void main(String[] args) throws Exception {
        basicYake();
    }
}
