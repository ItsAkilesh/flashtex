// name: Dl3Framing.cs
// purpose: Frame I/O for the display-list-v3 engine-host protocol
//   (docs/protocol/display-list-v3.md section 2): a little-endian u32 length
//   covering the kind byte and the body, then the kind byte, then the body.
//   Knows nothing about message contents; EngineHostClient interprets frames.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using System.Buffers.Binary;

namespace FlashTeX.EngineHost;

/// <summary>Frame kinds of display-list-v3 (spec section 2).</summary>
public static class Dl3Kind
{
    public const byte ClientHello = 0x01;
    public const byte Compile = 0x02;
    public const byte Cancel = 0x03;
    public const byte Bye = 0x04;
    public const byte HostHello = 0x41;
    public const byte Started = 0x42;
    public const byte Font = 0x43;
    public const byte Image = 0x44;
    public const byte Page = 0x45;
    public const byte Form = 0x46;
    public const byte Sources = 0x47;
    public const byte Diagnostic = 0x48;
    public const byte Done = 0x49;
    public const byte Error = 0x4A;
    public const byte Pages = 0x4B;
}

/// <summary>One protocol frame: its kind and its body bytes.</summary>
public readonly record struct Dl3Frame(byte Kind, byte[] Body);

/// <summary>A stream that breaks the framing rules; the connection is unusable after it.</summary>
public sealed class Dl3ProtocolException(string message) : Exception(message);

/// <summary>Reads and writes <see cref="Dl3Frame"/>s on a byte stream.</summary>
public static class Dl3Framing
{
    /// <summary>Largest legal value of the length field (2^28).</summary>
    public const int MaxFrameLength = 1 << 28;

    private const int LengthPrefixBytes = sizeof(uint);

    /// <summary>
    /// Reads one frame. Returns null on a clean end of stream at a frame boundary;
    /// an end of stream inside a frame, a zero length or an oversized length throws.
    /// </summary>
    public static async Task<Dl3Frame?> ReadAsync(Stream stream, CancellationToken ct)
    {
        var prefix = new byte[LengthPrefixBytes];
        if (!await TryReadExactAsync(stream, prefix, ct).ConfigureAwait(false))
        {
            return null;
        }

        var length = BinaryPrimitives.ReadUInt32LittleEndian(prefix);
        if (length == 0 || length > MaxFrameLength)
        {
            throw new Dl3ProtocolException($"corrupt frame length {length}");
        }

        var payload = new byte[length];
        if (!await TryReadExactAsync(stream, payload, ct).ConfigureAwait(false))
        {
            throw new Dl3ProtocolException("stream ended inside a frame");
        }

        return new Dl3Frame(payload[0], payload[1..]);
    }

    /// <summary>Writes one frame as a single buffer, so frames never interleave.</summary>
    public static async Task WriteAsync(Stream stream, byte kind, ReadOnlyMemory<byte> body, CancellationToken ct)
    {
        var length = body.Length + 1;
        if (length > MaxFrameLength)
        {
            throw new Dl3ProtocolException($"frame of {length} bytes exceeds the protocol limit");
        }

        var buffer = new byte[LengthPrefixBytes + length];
        BinaryPrimitives.WriteUInt32LittleEndian(buffer, (uint)length);
        buffer[LengthPrefixBytes] = kind;
        body.Span.CopyTo(buffer.AsSpan(LengthPrefixBytes + 1));
        await stream.WriteAsync(buffer, ct).ConfigureAwait(false);
        await stream.FlushAsync(ct).ConfigureAwait(false);
    }

    private static async Task<bool> TryReadExactAsync(Stream stream, byte[] buffer, CancellationToken ct)
    {
        var filled = 0;
        while (filled < buffer.Length)
        {
            var read = await stream.ReadAsync(buffer.AsMemory(filled), ct).ConfigureAwait(false);
            if (read == 0)
            {
                if (filled == 0)
                {
                    return false;
                }

                throw new Dl3ProtocolException("stream ended inside a frame");
            }

            filled += read;
        }

        return true;
    }
}
