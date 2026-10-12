// Win32 ownership adapter; exact native 3DM + Rust-bound layer metadata pair.
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace OM9LayerTransfer {
public static class NativeClipboard {
    const string GeometryFormat="Rhino 5.0 3DM Clip global mem",MetadataFormat="OM9.LayerSession.v1";
    [DllImport("user32",SetLastError=true)] static extern bool OpenClipboard(IntPtr owner);
    [DllImport("user32")] static extern bool CloseClipboard();
    [DllImport("user32")] static extern bool EmptyClipboard();
    [DllImport("user32")] static extern IntPtr GetClipboardData(uint format);
    [DllImport("user32")] static extern IntPtr SetClipboardData(uint format,IntPtr handle);
    [DllImport("user32")] static extern uint EnumClipboardFormats(uint previous);
    [DllImport("user32")] public static extern uint GetClipboardSequenceNumber();
    [DllImport("user32",CharSet=CharSet.Unicode)] static extern uint RegisterClipboardFormat(string name);
    [DllImport("kernel32")] static extern UIntPtr GlobalSize(IntPtr handle);
    [DllImport("kernel32")] static extern IntPtr GlobalLock(IntPtr handle);
    [DllImport("kernel32")] static extern bool GlobalUnlock(IntPtr handle);
    [DllImport("kernel32")] static extern IntPtr GlobalAlloc(uint flags,UIntPtr size);
    [DllImport("kernel32")] static extern IntPtr GlobalFree(IntPtr handle);
    sealed class Lock:IDisposable {
        public Lock(){if(!OpenClipboard(Process.GetCurrentProcess().MainWindowHandle))throw new InvalidOperationException("Clipboard busy; try again");}
        public void Dispose(){CloseClipboard();}
    }
    static byte[] Read(uint format,int maximum) {
        IntPtr h=GetClipboardData(format);if(h==IntPtr.Zero)throw new InvalidOperationException("Missing clipboard format");
        ulong n=GlobalSize(h).ToUInt64();if(n==0||n>(ulong)maximum)throw new InvalidOperationException("Clipboard format is not bounded global memory");
        IntPtr p=GlobalLock(h);if(p==IntPtr.Zero)throw new InvalidOperationException("Cannot read clipboard global memory");
        try {byte[] value=new byte[(int)n];Marshal.Copy(p,value,0,value.Length);return value;}finally {GlobalUnlock(h);}
    }
    public static byte[][] Capture() {
        using(var guard=new Lock()) {uint before=GetClipboardSequenceNumber();
            byte[] g=Read(RegisterClipboardFormat(GeometryFormat),512*1024*1024),m=Read(RegisterClipboardFormat(MetadataFormat),256*1024*1024);
            if(before==0||before!=GetClipboardSequenceNumber())throw new InvalidOperationException("Clipboard changed during paired capture");
            return new byte[][]{g,m};}
    }
    sealed class Allocation:IDisposable {
        public IntPtr Handle;
        public Allocation(byte[] value){Handle=GlobalAlloc(0x0002,(UIntPtr)value.Length);if(Handle==IntPtr.Zero)throw new OutOfMemoryException();
            IntPtr p=GlobalLock(Handle);if(p==IntPtr.Zero){Dispose();throw new InvalidOperationException("Clipboard allocation lock failed");}
            try{Marshal.Copy(value,0,p,value.Length);}finally{GlobalUnlock(Handle);}}
        public void Publish(uint format){if(SetClipboardData(format,Handle)==IntPtr.Zero)throw new InvalidOperationException("Clipboard publication failed");Handle=IntPtr.Zero;}
        public void Dispose(){if(Handle!=IntPtr.Zero){GlobalFree(Handle);Handle=IntPtr.Zero;}}
    }
    public static void Publish(byte[] g,byte[] m) {
        if(g.Length>512*1024*1024||m.Length>256*1024*1024)throw new InvalidOperationException("Clipboard pair exceeds budget");
        using(var guard=new Lock())using(var geometry=new Allocation(g))using(var metadata=new Allocation(m)) {
            var backup=new Dictionary<uint,Allocation>();long total=0;
            try {
                uint format=0;while((format=EnumClipboardFormats(format))!=0) {
                    if(backup.Count>=256)throw new InvalidOperationException("Previous clipboard format budget exceeded");
                    byte[] value=Read(format,512*1024*1024);total+=value.Length;
                    if(total>512L*1024*1024)throw new InvalidOperationException("Previous clipboard byte budget exceeded");
                    backup.Add(format,new Allocation(value));
                }
                if(!EmptyClipboard())throw new InvalidOperationException("Cannot acquire clipboard ownership");
                try {geometry.Publish(RegisterClipboardFormat(GeometryFormat));metadata.Publish(RegisterClipboardFormat(MetadataFormat));}
                catch {if(!EmptyClipboard())throw new InvalidOperationException("Cannot restore previous clipboard ownership");
                    foreach(var row in backup)row.Value.Publish(row.Key);throw;}
                // Eager global-memory formats remain owned by Windows after this process exits.
            }finally{foreach(var row in backup)row.Value.Dispose();}
        }
    }
}
}
