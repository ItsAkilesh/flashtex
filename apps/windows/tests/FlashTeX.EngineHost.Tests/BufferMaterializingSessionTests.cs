// name: BufferMaterializingSessionTests.cs
// purpose: The buffer-writing decorator: files and nested folders appear under the
//   root before the inner compile runs, existing files are overwritten, a request
//   with no buffers still creates the root, and a path that escapes the root is
//   refused before anything is written or delegated.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28
// modified: 2026-10-28 -- scratch-root scoping.

namespace FlashTeX.EngineHost.Tests;

public sealed class BufferMaterializingSessionTests : IDisposable
{
    private readonly DirectoryInfo _temp = Directory.CreateTempSubdirectory("fxmat");

    private sealed class RecordingSession : IEngineHostSession
    {
        public bool WasCalled { get; private set; }

        public Func<bool>? OnCall { get; set; }

        public Task<CompileResult> CompileAsync(CompileRequest request, Action<Dl3Frame>? onFrame, CancellationToken ct)
        {
            WasCalled = true;
            OnCall?.Invoke();
            var done = new CompileDone(request.Id, "ok", 0, 1, null, null, null, null, null, null);
            return Task.FromResult(new CompileResult(done, [], 0));
        }
    }

    private BufferMaterializingSession Session(IEngineHostSession inner) =>
        new(inner, Path.Combine(_temp.FullName, "proj"));

    public void Dispose() => _temp.Delete(recursive: true);

    private CompileRequest Request(params BufferFile[] buffers) =>
        new() { Id = 1, Root = Path.Combine(_temp.FullName, "proj"), Main = "main.tex", Buffers = buffers };

    [Fact]
    public async Task BuffersAreOnDiskWhenTheInnerCompileRuns()
    {
        var inner = new RecordingSession();
        var request = Request(new BufferFile("main.tex", "A"), new BufferFile("chapters/one.tex", "B"));
        var seenOnDisk = false;
        inner.OnCall = () => seenOnDisk = File.ReadAllText(Path.Combine(request.Root, "chapters", "one.tex")) == "B";

        await Session(inner).CompileAsync(request, null, CancellationToken.None);

        Assert.True(inner.WasCalled);
        Assert.True(seenOnDisk);
        Assert.Equal("A", File.ReadAllText(Path.Combine(request.Root, "main.tex")));
    }

    [Fact]
    public async Task ExistingFileIsOverwrittenWithTheNewText()
    {
        var request = Request(new BufferFile("main.tex", "new"));
        Directory.CreateDirectory(request.Root);
        await File.WriteAllTextAsync(Path.Combine(request.Root, "main.tex"), "old");

        await Session(new RecordingSession()).CompileAsync(request, null, CancellationToken.None);

        Assert.Equal("new", File.ReadAllText(Path.Combine(request.Root, "main.tex")));
    }

    [Fact]
    public async Task RealProjectRootIsNeverWritten()
    {
        var inner = new RecordingSession();
        var realRoot = Path.Combine(_temp.FullName, "real");
        Directory.CreateDirectory(realRoot);
        await File.WriteAllTextAsync(Path.Combine(realRoot, "main.tex"), "on disk");
        var request = Request(new BufferFile("main.tex", "unsaved edit")) with { Root = realRoot };

        await Session(inner).CompileAsync(request, null, CancellationToken.None);

        Assert.True(inner.WasCalled);
        Assert.Equal("on disk", File.ReadAllText(Path.Combine(realRoot, "main.tex")));
    }

    [Fact]
    public async Task NoBuffersStillCreatesTheRootAndCompiles()
    {
        var inner = new RecordingSession();
        var request = Request();

        await Session(inner).CompileAsync(request, null, CancellationToken.None);

        Assert.True(Directory.Exists(request.Root));
        Assert.True(inner.WasCalled);
    }

    [Theory]
    [InlineData("../escape.tex")]
    [InlineData("..\\escape.tex")]
    [InlineData("sub/../../escape.tex")]
    public async Task PathEscapingTheRootIsRefusedAndNothingIsWrittenOrCompiled(string badPath)
    {
        var inner = new RecordingSession();
        var request = Request(new BufferFile(badPath, "x"));

        var error = await Assert.ThrowsAsync<EngineHostException>(
            () => Session(inner).CompileAsync(request, null, CancellationToken.None));

        Assert.Equal("path", error.Code);
        Assert.False(inner.WasCalled);
        Assert.False(File.Exists(Path.Combine(_temp.FullName, "escape.tex")));
    }

    [Fact]
    public async Task SiblingFolderWithTheRootNameAsPrefixIsNotInsideTheRoot()
    {
        var request = Request(new BufferFile("../proj-evil/x.tex", "x"));

        await Assert.ThrowsAsync<EngineHostException>(
            () => Session(new RecordingSession()).CompileAsync(request, null, CancellationToken.None));
    }
}
