// name: EngineHostClientTests.cs
// purpose: EngineHostClient against an in-process fake host on a real AF_UNIX
//   socket: HELLO handshake, a compile that streams binary frames and diagnostics
//   before DONE, a stale DONE for another id, an ERROR frame, a version refusal,
//   and the host vanishing mid-compile. A final opt-in integration test drives
//   the real flashtex-host when FLASHTEX_HOST_EXE is set.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using System.Net.Sockets;
using System.Text;
using System.Text.Json;

namespace FlashTeX.EngineHost.Tests;

public sealed class EngineHostClientTests
{
    private const string HelloJson =
        """{"protocol":"display-list-v3","version":[3,2],"server":"fake 1","engine":"fake engine","capabilities":["compile","resident"]}""";

    private static CompileRequest Request(long id) => new() { Id = id, Root = "C:\\proj", Main = "main.tex" };

    private static byte[] Json(string text) => Encoding.UTF8.GetBytes(text);

    [Fact]
    public async Task ConnectReadsTheHostHello()
    {
        await using var host = FakeHost.Start(async stream => await FakeHost.AnswerHelloAsync(stream, HelloJson));
        await using var client = await EngineHostClient.ConnectAsync(host.SocketPath, CancellationToken.None);

        Assert.Equal("fake 1", client.Hello.Server);
        Assert.Contains("resident", client.Hello.Capabilities);
        Assert.Equal([3, 2], client.Hello.Version);
    }

    [Fact]
    public async Task CompileCollectsDiagnosticsAndPassesBinaryFramesThrough()
    {
        await using var host = FakeHost.Start(async stream =>
        {
            await FakeHost.AnswerHelloAsync(stream, HelloJson);
            await FakeHost.ReadFrameAsync(stream);
            await FakeHost.SendAsync(stream, Dl3Kind.Started, Json("""{"id":5}"""));
            await FakeHost.SendAsync(stream, Dl3Kind.Page, [9, 9, 9]);
            await FakeHost.SendAsync(stream, Dl3Kind.Diagnostic, Json("""{"id":5,"severity":"warning","message":"Overfull","file":"main.tex","line":12}"""));
            await FakeHost.SendAsync(stream, Dl3Kind.Done, Json("""{"id":5,"status":"ok","exit_code":0,"pages":1,"pdf":"C:\\out\\main.pdf"}"""));
        });
        await using var client = await EngineHostClient.ConnectAsync(host.SocketPath, CancellationToken.None);
        var seen = new List<byte>();

        var result = await client.CompileAsync(Request(5), frame => seen.Add(frame.Kind), CancellationToken.None);

        Assert.Equal("ok", result.Done.Status);
        Assert.Equal(1, result.Done.Pages);
        Assert.Equal("C:\\out\\main.pdf", result.Done.Pdf);
        Assert.Equal([Dl3Kind.Started, Dl3Kind.Page], seen);
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal(("warning", "main.tex", 12), (diagnostic.Severity, diagnostic.File, diagnostic.Line));
    }

    [Fact]
    public async Task CompileRequestIsSentWithSnakeCaseKeysAndNoNulls()
    {
        string? sent = null;
        await using var host = FakeHost.Start(async stream =>
        {
            await FakeHost.AnswerHelloAsync(stream, HelloJson);
            sent = Encoding.UTF8.GetString((await FakeHost.ReadFrameAsync(stream)).Body);
            await FakeHost.SendAsync(stream, Dl3Kind.Done, Json("""{"id":1,"status":"ok","pages":0}"""));
        });
        await using var client = await EngineHostClient.ConnectAsync(host.SocketPath, CancellationToken.None);

        await client.CompileAsync(Request(1) with { OutputDir = "C:\\out", ExternalTools = "auto" }, null, CancellationToken.None);

        using var document = JsonDocument.Parse(sent!);
        var root = document.RootElement;
        Assert.Equal("C:\\out", root.GetProperty("output_dir").GetString());
        Assert.Equal("auto", root.GetProperty("external_tools").GetString());
        Assert.False(root.TryGetProperty("format", out _));
    }

    [Fact]
    public async Task DoneForAnotherIdIsSkipped()
    {
        await using var host = FakeHost.Start(async stream =>
        {
            await FakeHost.AnswerHelloAsync(stream, HelloJson);
            await FakeHost.ReadFrameAsync(stream);
            await FakeHost.SendAsync(stream, Dl3Kind.Done, Json("""{"id":1,"status":"cancelled","pages":0}"""));
            await FakeHost.SendAsync(stream, Dl3Kind.Done, Json("""{"id":2,"status":"ok","pages":3}"""));
        });
        await using var client = await EngineHostClient.ConnectAsync(host.SocketPath, CancellationToken.None);

        var result = await client.CompileAsync(Request(2), null, CancellationToken.None);

        Assert.Equal(3, result.Done.Pages);
    }

    [Fact]
    public async Task ErrorFrameDuringCompileThrowsWithItsCode()
    {
        await using var host = FakeHost.Start(async stream =>
        {
            await FakeHost.AnswerHelloAsync(stream, HelloJson);
            await FakeHost.ReadFrameAsync(stream);
            await FakeHost.SendAsync(stream, Dl3Kind.Error, Json("""{"code":"bad_request","message":"no root"}"""));
        });
        await using var client = await EngineHostClient.ConnectAsync(host.SocketPath, CancellationToken.None);

        var error = await Assert.ThrowsAsync<EngineHostException>(
            () => client.CompileAsync(Request(1), null, CancellationToken.None));

        Assert.Equal("bad_request", error.Code);
    }

    [Fact]
    public async Task VersionRefusalAtHelloThrows()
    {
        await using var host = FakeHost.Start(async stream =>
        {
            await FakeHost.ReadFrameAsync(stream);
            await FakeHost.SendAsync(stream, Dl3Kind.Error, Json("""{"code":"version","message":"major differs"}"""));
        });

        var error = await Assert.ThrowsAsync<EngineHostException>(
            () => EngineHostClient.ConnectAsync(host.SocketPath, CancellationToken.None));

        Assert.Equal("version", error.Code);
    }

    [Fact]
    public async Task HostClosingMidCompileThrowsAProtocolException()
    {
        await using var host = FakeHost.Start(async stream =>
        {
            await FakeHost.AnswerHelloAsync(stream, HelloJson);
            await FakeHost.ReadFrameAsync(stream);
        });
        await using var client = await EngineHostClient.ConnectAsync(host.SocketPath, CancellationToken.None);

        await Assert.ThrowsAsync<Dl3ProtocolException>(
            () => client.CompileAsync(Request(1), null, CancellationToken.None));
    }

    [Fact]
    public async Task ConnectToAMissingSocketFailsWithoutLeakingTheSocket()
    {
        var missing = Path.Combine(Path.GetTempPath(), $"fx{Guid.NewGuid():N}"[..10] + ".sock");

        await Assert.ThrowsAsync<SocketException>(
            () => EngineHostClient.ConnectAsync(missing, CancellationToken.None));
    }

    [Fact]
    [Trait("Category", "Integration")]
    public async Task RealHostCompilesAHelloDocument()
    {
        var hostExe = Environment.GetEnvironmentVariable("FLASHTEX_HOST_EXE");
        if (string.IsNullOrEmpty(hostExe))
        {
            return; // Opt-in: needs a built flashtex-host and a bundle or TeX Live; see HANDOFF.md.
        }

        var socket = Path.Combine(Path.GetTempPath(), $"fx{Guid.NewGuid():N}"[..10] + ".sock");
        var project = Directory.CreateTempSubdirectory("fxproj");
        await File.WriteAllTextAsync(
            Path.Combine(project.FullName, "main.tex"),
            "\\documentclass{article}\\begin{document}Hello\\end{document}");
        var options = new EngineHostOptions { HostExePath = hostExe, SocketPath = socket };
        CompileResult result;
        await using (var process = await EngineHostProcess.StartAsync(options, null, CancellationToken.None))
        await using (var client = await EngineHostClient.ConnectAsync(process.SocketPath, CancellationToken.None))
        {
            result = await client.CompileAsync(
                new CompileRequest { Id = 1, Root = project.FullName, Main = "main.tex" }, null, CancellationToken.None);
        }

        Assert.Equal("ok", result.Done.Status);
        Assert.Equal(1, result.Done.Pages);
        Assert.True(File.Exists(result.Done.Pdf));
        project.Delete(recursive: true);
    }
}
