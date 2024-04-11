using System.Drawing;
using ComputeSharp;

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

           var rtx= GraphicsDevice.GetDefault();






        }
    }
}