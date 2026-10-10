// Rhino5 adapter only. Rust owns validation, digest binding and palette merge.
using System;
using System.Runtime.InteropServices;
using System.Text;

namespace OM9LayerTransfer {
public static partial class NativeLayerApi {
    [StructLayout(LayoutKind.Sequential)] public struct Bytes {public IntPtr Data;public UIntPtr Length;}
    [DllImport(Dll,CallingConvention=CallingConvention.Cdecl)] static extern uint om9_rhino_layer_abi();
    [DllImport(Dll,CallingConvention=CallingConvention.Cdecl)] static extern uint om9_rhino_layer_validate(Bytes s,out ulong h);
    [DllImport(Dll,CallingConvention=CallingConvention.Cdecl)] static extern uint om9_rhino_layer_prepare(Bytes s,uint scope,Bytes g,out ulong h);
    [DllImport(Dll,CallingConvention=CallingConvention.Cdecl)] static extern uint om9_rhino_layer_receive(Bytes m,Bytes g,Bytes d,Bytes b,out ulong h);
    [DllImport(Dll,CallingConvention=CallingConvention.Cdecl)] static extern uint om9_rhino_layer_capture(Bytes m,Bytes g,out ulong h);
    [DllImport(Dll,CallingConvention=CallingConvention.Cdecl)] static extern uint om9_rhino_layer_persistent(Bytes input,uint encode,out ulong h);
    [DllImport(Dll,CallingConvention=CallingConvention.Cdecl)] static extern uint om9_rhino_layer_result(ulong h,IntPtr output,UIntPtr capacity,out UIntPtr needed);
    [DllImport(Dll,CallingConvention=CallingConvention.Cdecl)] static extern uint om9_rhino_layer_free(ulong h);
    [DllImport(Dll,CallingConvention=CallingConvention.Cdecl)] static extern uint om9_rhino_layer_check(Bytes before,Bytes current);
    sealed class Pin : IDisposable {
        GCHandle handle;public Bytes View;
        internal Pin(byte[] bytes) {handle=GCHandle.Alloc(bytes,GCHandleType.Pinned);View=new Bytes {Data=handle.AddrOfPinnedObject(),Length=(UIntPtr)bytes.Length};}
        public void Dispose(){if(handle.IsAllocated)handle.Free();}
    }
    public static void Check(uint code) {if(code!=0)throw new InvalidOperationException("OM9 Rust layer error "+code);}
    public static void VerifyAbi(){if(IntPtr.Size!=8||om9_rhino_layer_abi()!=1)throw new InvalidOperationException("OM9 requires matching x64 Rust layer ABI1");}
    static byte[] Result(ulong h) {
        try {
            UIntPtr needed;uint code=om9_rhino_layer_result(h,IntPtr.Zero,UIntPtr.Zero,out needed);
            if(code!=15)Check(code);ulong n=needed.ToUInt64();if(n>256UL*1024*1024)throw new InvalidOperationException("Rust result exceeds budget");
            byte[] bytes=new byte[(int)n];using(var p=new Pin(bytes)){Check(om9_rhino_layer_result(h,p.View.Data,p.View.Length,out needed));}
            if(needed.ToUInt64()!=n)throw new InvalidOperationException("Immutable Rust result changed");return bytes;
        } finally {Check(om9_rhino_layer_free(h));}
    }
    public static byte[] Validate(byte[] s){VerifyAbi();ulong h;using(var p=new Pin(s)){Check(om9_rhino_layer_validate(p.View,out h));}return Result(h);}
    public static byte[] Prepare(byte[] s,uint scope,byte[] g){VerifyAbi();ulong h;using(var a=new Pin(s))using(var b=new Pin(g)){Check(om9_rhino_layer_prepare(a.View,scope,b.View,out h));}return Result(h);}
    public static byte[] Receive(byte[] m,byte[] g,byte[] d,byte[] bindings){VerifyAbi();ulong h;using(var a=new Pin(m))using(var b=new Pin(g))using(var c=new Pin(d))using(var e=new Pin(bindings)){Check(om9_rhino_layer_receive(a.View,b.View,c.View,e.View,out h));}return Result(h);}
    public static byte[] Capture(byte[] m,byte[] g){VerifyAbi();ulong h;using(var a=new Pin(m))using(var b=new Pin(g)){Check(om9_rhino_layer_capture(a.View,b.View,out h));}return Result(h);}
    public static byte[] Persistent(byte[] facts,uint encode){ulong h;using(var a=new Pin(facts)){Check(om9_rhino_layer_persistent(a.View,encode,out h));}return Result(h);}
    public static void CheckCurrent(byte[] before,byte[] current){using(var a=new Pin(before))using(var b=new Pin(current)){Check(om9_rhino_layer_check(a.View,b.View));}}
}
}
