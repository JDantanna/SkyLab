using System;
using System.Drawing;
using System.IO;
using System.Net;
using System.Net.Http;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Forms;
using Microsoft.Web.WebView2.Core;
using Microsoft.Web.WebView2.WinForms;

internal static class Program {
    [DllImport("user32.dll")] private static extern bool SetProcessDPIAware();
    [STAThread] private static int Main(string[] args) {
        if (args.Length > 0 && args[0] == "--fetch-update") {
            try { Console.OutputEncoding=new System.Text.UTF8Encoding(false);Console.Write(Fetch(args[1]).GetAwaiter().GetResult()); return 0; }
            catch (Exception e) { Console.Error.Write(e.Message); return 1; }
        }
        if (args.Length > 0 && args[0] == "--runtime-check") {
            try { Console.Write(CoreWebView2Environment.GetAvailableBrowserVersionString()); return 0; }
            catch (Exception e) { Console.Error.Write(e.Message); return 1; }
        }
        Uri target;
        if (args.Length < 1 || !Uri.TryCreate(args[0], UriKind.Absolute, out target) || target.Scheme != "http" || target.Host != "127.0.0.1") return 2;
        try {
            CoreWebView2Environment.GetAvailableBrowserVersionString();
            SetProcessDPIAware();
            Application.EnableVisualStyles(); Application.SetCompatibleTextRenderingDefault(false);
            bool created;
            using (Mutex mutex = new Mutex(true, "Local\\SkyLab.Window." + target.Port, out created)) {
                if (!created) { ShellWindow.ActivateExisting(target.Port); return 0; }
                Application.Run(new ShellWindow(target, args.Length > 1 && args[1] == "setup"));
            }
            return 0;
        } catch (Exception e) {
            MessageBox.Show("Não foi possível abrir a janela do SkyLab. Verifique a instalação do Microsoft Edge WebView2 Runtime.\n\n" + e.Message, "SkyLab", MessageBoxButtons.OK, MessageBoxIcon.Error);
            return 1;
        }
    }
    private static async Task<string> Fetch(string source) {
        Uri uri;
        if (!Uri.TryCreate(source, UriKind.Absolute, out uri) || uri.Scheme != "https" || !String.IsNullOrEmpty(uri.UserInfo)) throw new Exception("A fonte de atualização deve usar HTTPS, sem credenciais no endereço.");
        ServicePointManager.SecurityProtocol = SecurityProtocolType.Tls12;
        using (var cancel = new CancellationTokenSource(TimeSpan.FromSeconds(12)))
        using (var handler = new HttpClientHandler { AllowAutoRedirect = false, UseCookies = false })
        using (var client = new HttpClient(handler)) {
            client.Timeout = TimeSpan.FromSeconds(12);
            using (var response = await client.GetAsync(uri, HttpCompletionOption.ResponseHeadersRead, cancel.Token)) {
                response.EnsureSuccessStatusCode();
                if (response.Content.Headers.ContentLength > 1048576) throw new Exception("Manifesto de atualização muito grande.");
                using (var stream = await response.Content.ReadAsStreamAsync())
                using (var output = new MemoryStream()) {
                    var buffer = new byte[8192]; int read;
                    while ((read = await stream.ReadAsync(buffer, 0, buffer.Length, cancel.Token)) != 0) { output.Write(buffer, 0, read); if (output.Length > 1048576) throw new Exception("Manifesto de atualização muito grande."); }
                    return System.Text.Encoding.UTF8.GetString(output.ToArray());
                }
            }
        }
    }
}

internal sealed class ShellWindow : Form {
    private readonly WebView2 browser = new WebView2();
    private readonly Uri target;
    private readonly Panel caption = new Panel();
    private readonly CaptionButton[] captionButtons;
    private bool allowClose, checkingClose, ready;
    private readonly Label loading = new Label();
    [DllImport("user32.dll")] private static extern bool ReleaseCapture();
    [DllImport("user32.dll")] private static extern IntPtr SendMessage(IntPtr h, int msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] private static extern bool SetProp(IntPtr h, string name, IntPtr value);
    [DllImport("user32.dll")] private static extern IntPtr GetProp(IntPtr h, string name);
    [DllImport("user32.dll")] private static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] private static extern bool IsIconic(IntPtr h);
    [DllImport("user32.dll")] private static extern bool ShowWindow(IntPtr h, int cmd);
    private delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] private static extern bool EnumWindows(EnumProc proc, IntPtr l);
    [DllImport("dwmapi.dll")] private static extern int DwmSetWindowAttribute(IntPtr h, int attribute, ref int value, int size);
    public static void ActivateExisting(int port) {
        for (int i=0;i<20;i++) {
            bool found=false;
            EnumWindows(delegate(IntPtr h, IntPtr l) { if (GetProp(h,"SkyLabPort").ToInt64()==port) { if(IsIconic(h))ShowWindow(h,9); SetForegroundWindow(h); found=true; return false; } return true; },IntPtr.Zero);
            if(found)return; Thread.Sleep(75);
        }
    }
    public ShellWindow(Uri uri, bool setup) {
        target=uri; Text=setup?"SkyLab — Instalação":"SkyLab";
        FormBorderStyle=FormBorderStyle.None; BackColor=Color.FromArgb(14,36,85);
        StartPosition=FormStartPosition.CenterScreen; Padding=new Padding(1);
        var area=Screen.PrimaryScreen.WorkingArea;double uiScale;
        using(var graphics=Graphics.FromHwnd(IntPtr.Zero)){uiScale=graphics.DpiX/96.0;}
        AutoScaleMode=AutoScaleMode.None;
        MinimumSize=new Size(Math.Min((int)Math.Round((setup?700:900)*uiScale),area.Width-16),Math.Min((int)Math.Round((setup?540:600)*uiScale),area.Height-16));
        Size=new Size(Math.Min((int)Math.Round((setup?920:1380)*uiScale),area.Width-30),Math.Min((int)Math.Round((setup?670:920)*uiScale),area.Height-30));MaximizedBounds=area;
        try { Icon=new Icon(Path.Combine(AppDomain.CurrentDomain.BaseDirectory,"app.ico")); } catch { }
        loading.Text="SkyLab";loading.ForeColor=Color.White;loading.Font=new Font("Segoe UI",22);loading.Dock=DockStyle.Fill;loading.TextAlign=ContentAlignment.MiddleCenter;
        Controls.Add(loading);browser.Dock=DockStyle.Fill;browser.DefaultBackgroundColor=BackColor;browser.Visible=false;Controls.Add(browser);browser.BringToFront();
        caption.BackColor=BackColor;caption.Anchor=AnchorStyles.Top|AnchorStyles.Right;captionButtons=new[]{new CaptionButton("minimize",this),new CaptionButton("maximize",this),new CaptionButton("close",this)};
        foreach(var button in captionButtons){var command=button.Command;button.Click+=delegate{RunCommand(command);};caption.Controls.Add(button);}
        Controls.Add(caption);using(var graphics=CreateGraphics()){LayoutCaption(setup?74:72,graphics.DpiX/96.0);}caption.BringToFront();
        LocationChanged+=delegate{if(WindowState==FormWindowState.Normal)MaximizedBounds=Screen.FromHandle(Handle).WorkingArea;};
        Shown+=Initialize;Resize+=delegate { caption.Invalidate(true);if(ready) browser.CoreWebView2.PostWebMessageAsString(WindowState==FormWindowState.Maximized?"maximized":"restored"); };
        FormClosing+=CheckClosing;
    }
    protected override void OnHandleCreated(EventArgs e) {
        base.OnHandleCreated(e);SetProp(Handle,"SkyLabPort",new IntPtr(target.Port));
        try { int dark=1;DwmSetWindowAttribute(Handle,20,ref dark,4);int round=2;DwmSetWindowAttribute(Handle,33,ref round,4);int blue=0x0055240e;DwmSetWindowAttribute(Handle,34,ref blue,4); } catch { }
    }
    private async void Initialize(object sender,EventArgs e) {
        try {
            ShowWindow(Handle,1);
            var folder=Environment.GetEnvironmentVariable("SKYLAB_WEBVIEW_PROFILE");
            if(String.IsNullOrEmpty(folder))folder=Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),"SkyTEM","SkyLab","WebView2");
            var env=await CoreWebView2Environment.CreateAsync(null,folder);
            await browser.EnsureCoreWebView2Async(env);
            browser.CoreWebView2.Settings.AreDefaultContextMenusEnabled=false;
            browser.CoreWebView2.Settings.IsStatusBarEnabled=false;
            browser.CoreWebView2.Settings.AreDevToolsEnabled=false;
            browser.CoreWebView2.Settings.IsZoomControlEnabled=false;
            browser.CoreWebView2.Settings.AreBrowserAcceleratorKeysEnabled=false;
            browser.CoreWebView2.Settings.IsSwipeNavigationEnabled=false;
            browser.CoreWebView2.NavigationStarting+=delegate(object s,CoreWebView2NavigationStartingEventArgs ev) { Uri next;if(!Uri.TryCreate(ev.Uri,UriKind.Absolute,out next)||next.GetLeftPart(UriPartial.Authority)!=target.GetLeftPart(UriPartial.Authority))ev.Cancel=true; };
            browser.CoreWebView2.NewWindowRequested+=delegate(object s,CoreWebView2NewWindowRequestedEventArgs ev){ev.Handled=true;};
            browser.CoreWebView2.DownloadStarting+=delegate(object s,CoreWebView2DownloadStartingEventArgs ev){
                var testFolder=Environment.GetEnvironmentVariable("SKYLAB_DOWNLOADS");
                if(!String.IsNullOrEmpty(testFolder)){
                    Directory.CreateDirectory(testFolder);var file=Path.GetFileName(ev.ResultFilePath);var path=Path.Combine(testFolder,file);int number=1;
                    while(File.Exists(path))path=Path.Combine(testFolder,Path.GetFileNameWithoutExtension(file)+" ("+(number++)+")"+Path.GetExtension(file));
                    ev.ResultFilePath=path;
                }
                ev.Handled=true;
            };
            browser.CoreWebView2.WindowCloseRequested+=delegate{allowClose=true;Close();};
            browser.CoreWebView2.WebMessageReceived+=Message;
            browser.CoreWebView2.DocumentTitleChanged+=delegate{Text=browser.CoreWebView2.DocumentTitle;};
            browser.CoreWebView2.NavigationCompleted+=delegate { ready=true;browser.Visible=true;loading.Visible=false;caption.BringToFront(); };
            browser.Source=target;
        } catch(Exception ex) { MessageBox.Show("Não foi possível carregar a interface.\n\n"+ex.Message,"SkyLab",MessageBoxButtons.OK,MessageBoxIcon.Error);allowClose=true;Close(); }
    }
    private void Message(object sender,CoreWebView2WebMessageReceivedEventArgs e) {
        Uri source;if(!Uri.TryCreate(e.Source,UriKind.Absolute,out source)||source.GetLeftPart(UriPartial.Authority)!=target.GetLeftPart(UriPartial.Authority))return;
        string command;try{command=e.TryGetWebMessageAsString();}catch{return;}
        if(command.StartsWith("caption:")){var parts=command.Split(':');double height,scale;if(parts.Length==3&&Double.TryParse(parts[1],System.Globalization.NumberStyles.Float,System.Globalization.CultureInfo.InvariantCulture,out height)&&Double.TryParse(parts[2],System.Globalization.NumberStyles.Float,System.Globalization.CultureInfo.InvariantCulture,out scale)&&height>=40&&height<=100&&scale>=0.5&&scale<=4)LayoutCaption(height,scale);return;}
        RunCommand(command);
    }
    private void LayoutCaption(double height,double scale){int width=(int)Math.Round(44*scale),pixels=(int)Math.Round(height*scale);caption.Bounds=new Rectangle(ClientSize.Width-width*3-1,1,width*3,pixels);for(int i=0;i<3;i++)captionButtons[i].Bounds=new Rectangle(i*width,0,width,pixels);caption.BringToFront();}
    private void RunCommand(string command){
        switch(command) {
            case "minimize":WindowState=FormWindowState.Minimized;break;
            case "maximize":WindowState=WindowState==FormWindowState.Maximized?FormWindowState.Normal:FormWindowState.Maximized;break;
            case "close":Close();break;
            case "drag":if(WindowState!=FormWindowState.Maximized){ReleaseCapture();SendMessage(Handle,0xA1,new IntPtr(2),IntPtr.Zero);}break;
        }
    }
    private async void CheckClosing(object sender,FormClosingEventArgs e) {
        if(allowClose||!ready)return;e.Cancel=true;if(checkingClose)return;checkingClose=true;
        try { string answer=await browser.ExecuteScriptAsync("window.skyLabCanClose ? window.skyLabCanClose() : true");if(answer=="true"){allowClose=true;Close();} }
        catch { allowClose=true;Close(); }
        finally {checkingClose=false;}
    }
    protected override void WndProc(ref System.Windows.Forms.Message m) {
        base.WndProc(ref m);
        if(m.Msg==0x84&&WindowState==FormWindowState.Normal) {
            Point p=PointToClient(new Point((short)(m.LParam.ToInt64()&0xffff),(short)((m.LParam.ToInt64()>>16)&0xffff)));
            int edge=6;bool left=p.X<edge,right=p.X>=Width-edge,top=p.Y<edge,bottom=p.Y>=Height-edge;
            if(top&&left)m.Result=new IntPtr(13);else if(top&&right)m.Result=new IntPtr(14);else if(bottom&&left)m.Result=new IntPtr(16);else if(bottom&&right)m.Result=new IntPtr(17);else if(left)m.Result=new IntPtr(10);else if(right)m.Result=new IntPtr(11);else if(top)m.Result=new IntPtr(12);else if(bottom)m.Result=new IntPtr(15);
        }
    }
}

internal sealed class CaptionButton : Button {
    public readonly string Command;
    private readonly Form owner;
    private bool hovering;
    public CaptionButton(string command,Form form){Command=command;owner=form;FlatStyle=FlatStyle.Flat;FlatAppearance.BorderSize=0;BackColor=Color.FromArgb(14,36,85);ForeColor=Color.FromArgb(230,237,249);UseVisualStyleBackColor=false;TabStop=false;AccessibleName=command=="minimize"?"Minimizar":command=="maximize"?"Maximizar ou restaurar":"Fechar janela";}
    protected override void OnMouseEnter(EventArgs e){base.OnMouseEnter(e);hovering=true;Invalidate();}
    protected override void OnMouseLeave(EventArgs e){base.OnMouseLeave(e);hovering=false;Invalidate();}
    protected override void OnPaint(PaintEventArgs e){
        e.Graphics.Clear(hovering?(Command=="close"?Color.FromArgb(184,71,84):Color.FromArgb(33,60,106)):BackColor);
        float scale=e.Graphics.DpiX/96f,size=10*scale,x=(Width-size)/2,y=(Height-size)/2;
        using(var pen=new Pen(ForeColor,Math.Max(1,scale))){
            if(Command=="minimize")e.Graphics.DrawLine(pen,x,y+size,x+size,y+size);
            else if(Command=="close"){e.Graphics.DrawLine(pen,x,y,x+size,y+size);e.Graphics.DrawLine(pen,x+size,y,x,y+size);}
            else if(owner.WindowState==FormWindowState.Maximized){e.Graphics.DrawRectangle(pen,x+3*scale,y-2*scale,size-2*scale,size-2*scale);using(var fill=new SolidBrush(hovering?Color.FromArgb(33,60,106):BackColor))e.Graphics.FillRectangle(fill,x,y+scale,size-2*scale,size-2*scale);e.Graphics.DrawRectangle(pen,x,y+scale,size-2*scale,size-2*scale);}
            else e.Graphics.DrawRectangle(pen,x,y,size,size);
        }
    }
}

