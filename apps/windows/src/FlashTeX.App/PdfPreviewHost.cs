// name: PdfPreviewHost.cs
// purpose: Preview pane for the new engine: draws the PDF flashtex-host writes
//   (DONE.pdf) with the built-in Windows.Data.Pdf renderer, one image per page in
//   a vertical scroller. The PDF is the engine's exact output, so no font or
//   layout work happens in the app. The file is read into memory first, because
//   the host rewrites it on every compile and a held handle would block that.
//   A newer Display call abandons an older one still rendering, and the scroll
//   position survives a refresh. Click-to-source is not available in this mode;
//   it returns with native display-list-v3 page drawing.
// author: Claude Sonnet 5.5 (paired with Akilesh S)
// date: 2026-10-28

using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Data.Pdf;
using Windows.Storage.Streams;

namespace FlashTeX.App;

/// <summary>Shows a PDF file as a column of rendered pages.</summary>
public sealed class PdfPreviewHost : UserControl
{
    private const double PageGapDips = 12;
    private const double DipsPerPoint = 96.0 / 72.0;
    private const double RenderPixelScale = 2.0;
    private const uint MaxPages = 400;
    private const double MaxFitScale = 1.25;
    private const double MinFitWidthDips = 120;
    private const double ScrollBarAllowanceDips = 18;

    private readonly ScrollViewer _scroller = new()
    {
        HorizontalScrollBarVisibility = ScrollBarVisibility.Auto,
        VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
        ZoomMode = ZoomMode.Disabled,
    };

    private readonly StackPanel _pages = new()
    {
        Spacing = PageGapDips,
        Padding = new Thickness(PageGapDips),
        HorizontalAlignment = HorizontalAlignment.Center,
    };

    private int _generation;

    public PdfPreviewHost()
    {
        _scroller.Content = _pages;
        _scroller.SizeChanged += (_, _) => FitPagesToWidth();
        Content = _scroller;
    }

    /// <summary>The last failure to draw, for the status bar; null when the last Display succeeded.</summary>
    public string? Failure { get; private set; }

    /// <summary>Draws the PDF at <paramref name="path"/>, replacing the pages shown now once all are ready.</summary>
    public async Task DisplayAsync(string path)
    {
        var generation = ++_generation;
        try
        {
            var document = await LoadAsync(path).ConfigureAwait(true);
            var images = await RenderPagesAsync(document, generation).ConfigureAwait(true);
            if (images is null || generation != _generation)
            {
                return;
            }

            var offset = _scroller.VerticalOffset;
            _pages.Children.Clear();
            foreach (var image in images)
            {
                _pages.Children.Add(image);
            }

            FitPagesToWidth();
            _scroller.ChangeView(null, offset, null, disableAnimation: true);
            Failure = null;
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or System.Runtime.InteropServices.COMException)
        {
            Failure = ex.Message;
        }
    }

    /// <summary>Sizes every page to the pane's width (never beyond MaxFitScale of its natural size), keeping its aspect ratio.</summary>
    private void FitPagesToWidth()
    {
        var available = Math.Max(MinFitWidthDips, _scroller.ActualWidth - (2 * PageGapDips) - ScrollBarAllowanceDips);
        foreach (var image in _pages.Children.OfType<Image>())
        {
            if (image.Tag is not Windows.Foundation.Size natural)
            {
                continue;
            }

            var scale = Math.Min(MaxFitScale, available / natural.Width);
            image.Width = natural.Width * scale;
            image.Height = natural.Height * scale;
        }
    }

    /// <summary>Removes every page, as when there is nothing to show.</summary>
    public void Clear()
    {
        _generation++;
        _pages.Children.Clear();
    }

    private static async Task<PdfDocument> LoadAsync(string path)
    {
        var bytes = await File.ReadAllBytesAsync(path).ConfigureAwait(true);
        var stream = new InMemoryRandomAccessStream();
        using (var writer = new DataWriter(stream))
        {
            writer.WriteBytes(bytes);
            await writer.StoreAsync();
            writer.DetachStream();
        }

        stream.Seek(0);
        return await PdfDocument.LoadFromStreamAsync(stream);
    }

    private async Task<List<Image>?> RenderPagesAsync(PdfDocument document, int generation)
    {
        var images = new List<Image>();
        var count = Math.Min(document.PageCount, MaxPages);
        for (uint index = 0; index < count; index++)
        {
            if (generation != _generation)
            {
                return null;
            }

            images.Add(await RenderPageAsync(document.GetPage(index)));
        }

        return images;
    }

    private static async Task<Image> RenderPageAsync(PdfPage page)
    {
        using var stream = new InMemoryRandomAccessStream();
        var options = new PdfPageRenderOptions
        {
            DestinationWidth = (uint)Math.Ceiling(page.Size.Width * DipsPerPoint * RenderPixelScale),
        };
        await page.RenderToStreamAsync(stream, options);
        var bitmap = new BitmapImage();
        await bitmap.SetSourceAsync(stream);
        return new Image
        {
            Source = bitmap,
            Width = page.Size.Width * DipsPerPoint,
            Height = page.Size.Height * DipsPerPoint,
            Stretch = Stretch.Uniform,
            Tag = new Windows.Foundation.Size(page.Size.Width * DipsPerPoint, page.Size.Height * DipsPerPoint),
        };
    }
}
