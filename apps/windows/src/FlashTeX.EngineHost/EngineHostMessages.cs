// name: EngineHostMessages.cs
// purpose: Typed JSON messages of the display-list-v3 engine-host protocol that
//   the Windows app reads or writes (spec sections 6.2-6.4): the client HELLO,
//   the host HELLO, COMPILE, DIAGNOSTIC and DONE. Page, font and image frames
//   are binary and stay as raw Dl3Frame values until a renderer needs them.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using System.Text.Json;
using System.Text.Json.Serialization;

namespace FlashTeX.EngineHost;

/// <summary>A file as the editor holds it, sent so the host compiles unsaved text (3.1 buffers).</summary>
public sealed record BufferFile(string Path, string Text);

/// <summary>The COMPILE message (spec section 6.3). Paths are absolute where the spec says so.</summary>
public sealed record CompileRequest
{
    public required long Id { get; init; }

    public required string Root { get; init; }

    public required string Main { get; init; }

    public string? Format { get; init; }

    public string? OutputDir { get; init; }

    public string? Jobname { get; init; }

    public string? ShellEscape { get; init; }

    public string? ExternalTools { get; init; }

    public bool? Incremental { get; init; }

    public int? Viewport { get; init; }

    public IReadOnlyList<BufferFile>? Buffers { get; init; }
}

/// <summary>The host's HELLO answer (spec section 6.2); keys this app does not use are ignored.</summary>
public sealed record HostHello(
    string Protocol,
    int[] Version,
    string Server,
    string Engine,
    IReadOnlyList<string> Capabilities);

/// <summary>A DIAGNOSTIC message (spec section 6.4).</summary>
public sealed record HostDiagnostic(long Id, string Severity, string Message, string? File, int? Line);

/// <summary>The DONE message that ends one compile (spec section 6.4).</summary>
public sealed record CompileDone(
    long Id,
    string Status,
    int? ExitCode,
    int Pages,
    long? Bytes,
    string? Pdf,
    string? Log,
    double? ElapsedMs,
    double? FirstPageMs,
    string? Mode);

/// <summary>The host reported an ERROR frame, or the handshake was refused.</summary>
public sealed class EngineHostException(string code, string message) : Exception($"{code}: {message}")
{
    public string Code { get; } = code;
}

/// <summary>Shared JSON options: snake_case keys, nulls omitted, unknown keys ignored.</summary>
internal static class Dl3Json
{
    public static readonly JsonSerializerOptions Options = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        PropertyNameCaseInsensitive = true,
    };

    public static byte[] Serialize<T>(T value) => JsonSerializer.SerializeToUtf8Bytes(value, Options);

    public static T Deserialize<T>(byte[] body) =>
        JsonSerializer.Deserialize<T>(body, Options)
        ?? throw new Dl3ProtocolException($"empty {typeof(T).Name} message");
}
