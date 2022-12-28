using SharpDX.DXGI;
using SharpDX.Direct2D1;
using SharpDX.DirectWrite;
using SharpDX.IO;
using SharpDX.WIC;
using System.Drawing;
using Factory = SharpDX.Direct2D1.Factory;
using PixelFormat = SharpDX.Direct2D1.PixelFormat;
using AlphaMode = SharpDX.Direct2D1.AlphaMode;
using SharpDX.Mathematics.Interop;

namespace DXtest
{
    public partial class Form1 : Form
    {
        public Form1()
        {
            InitializeComponent();

            this.Click += Form1_Click;



        }

        private void Form1_Click(object? sender, EventArgs e)
        {
            
            Factory factory = new Factory();

            var p = new PixelFormat(Format.B8G8R8A8_UNorm, AlphaMode.Ignore);

            var h = new HwndRenderTargetProperties
            {
                Hwnd = this.Handle,
                PixelSize = new SharpDX.Size2(this.Width, this.Height),
                PresentOptions = PresentOptions.None,
            };

            var r = new RenderTargetProperties(RenderTargetType.Hardware, p, 0, 0, RenderTargetUsage.None, FeatureLevel.Level_DEFAULT);

            var render = new WindowRenderTarget(factory, r, h);

            render.AntialiasMode = AntialiasMode.PerPrimitive;
            render.BeginDraw();

            var brush = new SolidColorBrush(render, new RawColor4(1f, 0, 0, 1));
            render.FillEllipse(new Ellipse(new RawVector2(0, 0), 50, 50), brush);

            render.EndDraw();
        }
    }
}