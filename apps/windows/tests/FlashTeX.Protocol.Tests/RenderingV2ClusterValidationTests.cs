// name: RenderingV2ClusterValidationTests.cs
// purpose: Pins the display-list-v2 validator's rule about clusters that carry no glyph.
//   TeX sets an interword space as glue, so a space INSIDE a glyph run is a positioned
//   cluster with no glyph at all; refusing those made every real document containing
//   `\` at end of line fall back to the runtime-v1 renderer. A cluster with visible text
//   and no glyph is still refused, because painting that run would drop a real character.
// author: Claude Opus 5
// date: 2026-09-19

using FlashTeX.Protocol.RenderingV2;

namespace FlashTeX.Protocol.Tests;

public class RenderingV2ClusterValidationTests
{
    private const long Point = Ticks.TicksPerPoint;
    private const string Hex64 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    [Theory]
    [InlineData(" ")]   // an ordinary interword space that fell inside a run
    [InlineData("\r")]  // what `\` at end of line actually reaches the logical text as
    [InlineData("\t")]
    public void AWhitespaceClusterMayCarryNoGlyph(string separator)
    {
        RenderingV2Protocol.Validate(ListWithSeparator(separator, giveSeparatorAGlyph: false));
    }

    [Fact]
    public void AVisibleClusterWithNoGlyphIsStillRefused()
    {
        var thrown = Assert.Throws<RenderingV2ValidationException>(
            () => RenderingV2Protocol.Validate(ListWithSeparator("-", giveSeparatorAGlyph: false)));
        Assert.Equal("invalid_display_list", thrown.Code);
        Assert.Contains("have no glyph", thrown.Message, StringComparison.Ordinal);
        Assert.Contains("cluster 1", thrown.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void AVisibleClusterWithAGlyphIsAccepted()
    {
        RenderingV2Protocol.Validate(ListWithSeparator("-", giveSeparatorAGlyph: true));
    }

    /// <summary>
    /// A one-page list whose single run reads <c>a{separator}b</c> as three clusters, with
    /// the middle one optionally left without a glyph of its own.
    /// </summary>
    private static DisplayList ListWithSeparator(string separator, bool giveSeparatorAGlyph)
    {
        string text = "a" + separator + "b";
        int separatorBytes = ByteOffsets.Utf8ByteCount(separator);
        var glyphs = new List<Glyph>
        {
            new(10, 0, 100 * Point, 5 * Point, 0, 0),
        };
        if (giveSeparatorAGlyph)
        {
            glyphs.Add(new Glyph(11, 5 * Point, 100 * Point, 3 * Point, 0, 1));
        }
        glyphs.Add(new Glyph(12, 8 * Point, 100 * Point, 5 * Point, 0, 2));

        var clusters = new List<Cluster>
        {
            Cluster(0, 1, 0),
            Cluster(1, 1 + separatorBytes, 5),
            Cluster(1 + separatorBytes, 1 + separatorBytes + 1, 8),
        };
        var run = new GlyphRun("f1", 10 * Point, text, glyphs, clusters, Paint.Black);
        var page = new Page(1, 612 * Point, 792 * Point, new[] { new Item.OfGlyphRun(run) });
        return new DisplayList(
            "p",
            1,
            new[] { "glyph_run", "rgba-srgb", "cluster-actualtext" },
            new[] { new DocumentResource("main.tex", 1, Hex64, 16) },
            new[] { new FontResource("f1", Hex64, 1000, "opentype-cff", 0, 1000, 100, "Test") },
            new[] { page },
            Array.Empty<Diagnostic>());
    }

    private static Cluster Cluster(int startByte, int endByte, long x) => new(
        startByte,
        endByte,
        new[] { new Rect(x * Point, 95 * Point, 3 * Point, 10 * Point) },
        new[] { new Caret(startByte, x * Point, 95 * Point, 10 * Point) },
        null,
        "test fixture");
}
