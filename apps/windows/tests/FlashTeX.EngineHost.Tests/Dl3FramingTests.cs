// name: Dl3FramingTests.cs
// purpose: Happy-path, edge and failure cases for display-list-v3 frame I/O:
//   round trip, empty body, back-to-back frames, clean EOF, truncated frames,
//   zero and oversized lengths (spec section 2).
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using System.Buffers.Binary;

namespace FlashTeX.EngineHost.Tests;

public sealed class Dl3FramingTests
{
    private static readonly CancellationToken Ct = CancellationToken.None;

    private static MemoryStream Bytes(params byte[] bytes) => new(bytes);

    [Fact]
    public async Task WrittenFrameReadsBackWithKindAndBody()
    {
        using var stream = new MemoryStream();
        await Dl3Framing.WriteAsync(stream, Dl3Kind.Compile, new byte[] { 1, 2, 3 }, Ct);
        stream.Position = 0;

        var frame = await Dl3Framing.ReadAsync(stream, Ct);

        Assert.NotNull(frame);
        Assert.Equal(Dl3Kind.Compile, frame.Value.Kind);
        Assert.Equal(new byte[] { 1, 2, 3 }, frame.Value.Body);
    }

    [Fact]
    public async Task LengthPrefixIsLittleEndianAndCountsTheKindByte()
    {
        using var stream = new MemoryStream();
        await Dl3Framing.WriteAsync(stream, Dl3Kind.Bye, new byte[] { 7, 7 }, Ct);

        var raw = stream.ToArray();

        Assert.Equal(3u, BinaryPrimitives.ReadUInt32LittleEndian(raw));
        Assert.Equal(Dl3Kind.Bye, raw[4]);
    }

    [Fact]
    public async Task EmptyBodyIsLegal()
    {
        using var stream = new MemoryStream();
        await Dl3Framing.WriteAsync(stream, Dl3Kind.Bye, ReadOnlyMemory<byte>.Empty, Ct);
        stream.Position = 0;

        var frame = await Dl3Framing.ReadAsync(stream, Ct);

        Assert.Empty(frame!.Value.Body);
    }

    [Fact]
    public async Task BackToBackFramesComeOutInOrderThenCleanEof()
    {
        using var stream = new MemoryStream();
        await Dl3Framing.WriteAsync(stream, Dl3Kind.Started, new byte[] { 1 }, Ct);
        await Dl3Framing.WriteAsync(stream, Dl3Kind.Done, new byte[] { 2 }, Ct);
        stream.Position = 0;

        Assert.Equal(Dl3Kind.Started, (await Dl3Framing.ReadAsync(stream, Ct))!.Value.Kind);
        Assert.Equal(Dl3Kind.Done, (await Dl3Framing.ReadAsync(stream, Ct))!.Value.Kind);
        Assert.Null(await Dl3Framing.ReadAsync(stream, Ct));
    }

    [Fact]
    public async Task ZeroLengthIsCorrupt()
    {
        using var stream = Bytes(0, 0, 0, 0);

        await Assert.ThrowsAsync<Dl3ProtocolException>(() => Dl3Framing.ReadAsync(stream, Ct));
    }

    [Fact]
    public async Task LengthOverTheLimitIsCorruptAndNeverAllocated()
    {
        var prefix = new byte[4];
        BinaryPrimitives.WriteUInt32LittleEndian(prefix, (uint)Dl3Framing.MaxFrameLength + 1);
        using var stream = new MemoryStream(prefix);

        await Assert.ThrowsAsync<Dl3ProtocolException>(() => Dl3Framing.ReadAsync(stream, Ct));
    }

    [Fact]
    public async Task StreamEndingInsideThePrefixThrows()
    {
        using var stream = Bytes(5, 0);

        await Assert.ThrowsAsync<Dl3ProtocolException>(() => Dl3Framing.ReadAsync(stream, Ct));
    }

    [Fact]
    public async Task StreamEndingInsideTheBodyThrows()
    {
        using var stream = Bytes(4, 0, 0, 0, Dl3Kind.Done, 1);

        await Assert.ThrowsAsync<Dl3ProtocolException>(() => Dl3Framing.ReadAsync(stream, Ct));
    }
}
