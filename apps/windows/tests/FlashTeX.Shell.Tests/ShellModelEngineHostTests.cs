// name: ShellModelEngineHostTests.cs
// purpose: ShellModel's engine-host compile path (ShellModel.EngineHost.cs) against a
//   fake IEngineHostSession: the COMPILE it builds (root, relative `/` paths, unsaved
//   buffers), the DONE it applies (status, diagnostics with file:line, PDF path,
//   render generation) and its failure cases (host error, document outside the root,
//   nothing attached). No process or socket is involved.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using FlashTeX.EngineHost;
using FlashTeX.Protocol.RuntimeV1;

namespace FlashTeX.Shell.Tests;

public sealed class ShellModelEngineHostTests
{
    private const string Scratch = @"C:\scratch\proj";
    private const string OutputDir = @"C:\scratch\out";

    private sealed class FakeSession : IEngineHostSession
    {
        public List<FlashTeX.EngineHost.CompileRequest> Requests { get; } = [];

        public Func<FlashTeX.EngineHost.CompileRequest, FlashTeX.EngineHost.CompileResult> Respond { get; set; } =
            request => Ok(request.Id);

        public Task<FlashTeX.EngineHost.CompileResult> CompileAsync(
            FlashTeX.EngineHost.CompileRequest request, Action<Dl3Frame>? onFrame, CancellationToken ct)
        {
            Requests.Add(request);
            return Task.FromResult(Respond(request));
        }
    }

    private static FlashTeX.EngineHost.CompileResult Ok(long id, params HostDiagnostic[] diagnostics) =>
        new(new CompileDone(id, "ok", 0, 2, 1000, @"C:\scratch\out\main.pdf", null, 12.5, 3.1, "incremental"),
            diagnostics, 5);

    private static async Task<(ShellModel Model, FakeSession Session)> AttachedAsync()
    {
        var model = new ShellModel(compileDebounceInterval: TimeSpan.Zero);
        var session = new FakeSession();
        model.AttachEngineHost(session, Scratch, OutputDir);
        await model.OpenDocumentAsync("main.tex", "Hello\n");
        return (model, session);
    }

    [Fact]
    public async Task CompileSendsScratchRootRelativeMainAndEveryDocumentAsBuffers()
    {
        var (model, session) = await AttachedAsync();
        await model.OpenDocumentAsync("chapters/one.tex", "One\n");
        model.SwitchActiveDocument("main.tex");

        await model.CompileNowAsync();

        var request = Assert.Single(session.Requests);
        Assert.Equal(Scratch, request.Root);
        Assert.Equal("main.tex", request.Main);
        Assert.Equal(OutputDir, request.OutputDir);
        Assert.Equal("auto", request.ExternalTools);
        Assert.Equal(["main.tex", "chapters/one.tex"], request.Buffers!.Select(b => b.Path));
        Assert.Equal("Hello\n", request.Buffers![0].Text);
    }

    [Fact]
    public async Task UnsavedEditIsInTheBufferOfTheNextCompile()
    {
        var (model, session) = await AttachedAsync();
        model.UpdateDocumentText("main.tex", "Edited\n");

        await model.CompileNowAsync();

        Assert.Equal("Edited\n", session.Requests.Last().Buffers!.Single().Text);
    }

    [Fact]
    public async Task ProjectRootWinsOverTheScratchRootAndAbsolutePathsBecomeRelative()
    {
        var (model, session) = await AttachedAsync();
        model.ProjectRoot = @"D:\work\paper";
        await model.OpenDocumentAsync(@"D:\work\paper\sections\intro.tex", "Intro\n");

        await model.CompileNowAsync();

        var request = session.Requests.Last();
        Assert.Equal(@"D:\work\paper", request.Root);
        Assert.Equal("sections/intro.tex", request.Main);
        Assert.Equal(["main.tex", "sections/intro.tex"], request.Buffers!.Select(b => b.Path));
    }

    [Fact]
    public async Task DoneIsAppliedToStatusPdfPathAndRenderGeneration()
    {
        var (model, _) = await AttachedAsync();
        var before = model.RenderGeneration;

        await model.CompileNowAsync();

        Assert.True(model.HasCompileResult);
        Assert.Equal(@"C:\scratch\out\main.pdf", model.LastPdfPath);
        Assert.Equal(before + 1, model.RenderGeneration);
        Assert.Contains("2 page(s)", model.WorkerStatus, StringComparison.Ordinal);
        Assert.Contains("incremental", model.WorkerStatus, StringComparison.Ordinal);
        Assert.Null(model.LastDisplayList);
    }

    [Fact]
    public async Task DiagnosticsKeepSeverityAndCarryFileAndLine()
    {
        var (model, session) = await AttachedAsync();
        session.Respond = request => Ok(
            request.Id,
            new HostDiagnostic(request.Id, "error", "Undefined control sequence.", "main.tex", 7),
            new HostDiagnostic(request.Id, "warning", "Overfull hbox", null, null));

        await model.CompileNowAsync();

        Assert.Equal(1, model.ErrorCount);
        Assert.Equal(1, model.WarningCount);
        Assert.Equal("main.tex:7: Undefined control sequence.", model.Diagnostics[0].Message);
        Assert.Equal("Overfull hbox", model.Diagnostics[1].Message);
    }

    [Fact]
    public async Task HostErrorBecomesAStatusAndClearsThePdfPath()
    {
        var (model, session) = await AttachedAsync();
        await model.CompileNowAsync();
        session.Respond = _ => throw new EngineHostException("exited", "host died");

        await model.CompileNowAsync();

        Assert.Null(model.LastPdfPath);
        Assert.Contains("compile failed", model.WorkerStatus, StringComparison.Ordinal);
        Assert.Contains("host died", model.WorkerStatus, StringComparison.Ordinal);
    }

    [Fact]
    public async Task ActiveDocumentOutsideTheRootIsSkippedWithoutCallingTheHost()
    {
        var (model, session) = await AttachedAsync();
        model.ProjectRoot = @"D:\work\paper";
        await model.OpenDocumentAsync(@"E:\elsewhere\note.tex", "Note\n");

        await model.CompileNowAsync();

        Assert.Empty(session.Requests);
        Assert.Contains("outside the project folder", model.WorkerStatus, StringComparison.Ordinal);
    }

    [Fact]
    public async Task EngineCountsAsAttachedAndDetachingClearsIt()
    {
        var (model, session) = await AttachedAsync();

        Assert.True(model.HasWorkerAttached);
        Assert.True(model.HasEngineAttached);

        model.DetachEngineHost();
        await model.CompileNowAsync();

        Assert.False(model.HasWorkerAttached);
        Assert.Empty(session.Requests);
    }

    [Fact]
    public async Task SecondCompileUsesANewIncreasingId()
    {
        var (model, session) = await AttachedAsync();

        await model.CompileNowAsync();
        await model.CompileNowAsync();

        Assert.Equal(2, session.Requests.Select(r => r.Id).Distinct().Count());
        Assert.True(session.Requests[1].Id > session.Requests[0].Id);
    }
}
