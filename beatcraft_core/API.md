# Required interfaces:

```java

class BeatcraftCore {
    void init();

}

/// describes what sets and diffs are available
class SetInfo {
    String characteristic;
    DifficultyInfo[] difficulties;
}

/// Describes each difficulty
class DifficultyInfo {
    String characteristic;
    String difficulty;
    @Nullable String label;
    String[] mappers;
    String[] lighters;

    BeatmapController load() throws IOException;
}

class Info {
    private long handle;
    static Info load(String path) throws IOException;

    String getTitle();
    String getSubtitle();
    String getAuthor();

    float getDuration();
    float getBpm();

    SetInfo[] getSetInfo();
}

class BeatmapController {
    private long handle;
    Vector2f getOrigin();
    void setOrigin(Vector2f origin);

}

class RuntimeData {
    private long handle;
    float getNjs();
    float getBpm();
    float getSpawnOffset();
    ColorScheme getColorScheme();
}

class ColorScheme {
    private long handle;
    long getLeftNote();
    long getRightNote();
    long getObstacle();
    long getEnvironmentPrimary();
    long getEnvironmentSecondary();
    long getEnvironmentWhite();
    long getBoostPrimary();
    long getBoostSecondary();
    long getBoostWhite();
    // Setters?
}

class GameObject {
    private long handle;
    float getBeat();
    Vector2f getGridPos(Vector2f out);
    @Nullable Matrix4f animateComplex(
        Matrix4f base, // value will be modified
        float beat,
        RuntimeData data,
    );
}






```





