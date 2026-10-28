using System;
using System.Drawing;
using System.Windows.Forms;

internal static class NativeStartup {
    [STAThread] private static int Main() {
        try {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            foreach(bool setup in new[]{false,true}) {
                using(var window=new ShellWindow(new Uri("http://127.0.0.1:18787"),setup)) {
                    PictureBox splash=null;
                    foreach(Control control in window.Controls)if(control is PictureBox)splash=(PictureBox)control;
                    if(splash==null||splash.Image==null||splash.Image.Width!=128||splash.Image.Height!=128)
                        throw new Exception("Startup logo did not load at 128 pixels.");
                    using(var image=new Bitmap(splash.Image)) {
                        bool visible=false;
                        for(int y=0;y<image.Height&&!visible;y++)for(int x=0;x<image.Width;x++)if(image.GetPixel(x,y).A>0){visible=true;break;}
                        if(!visible)throw new Exception("Startup logo is empty.");
                    }
                    splash.Image.Dispose();splash.Image=null;
                    Console.WriteLine((setup?"Installer":"Application")+": native window and startup logo OK");
                }
            }
            return 0;
        } catch(Exception e) { Console.Error.WriteLine(e);return 1; }
    }
}
