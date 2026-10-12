using System;
using System.IO;
using System.Text;
using System.Web.Script.Serialization;
using System.Collections.Generic;
using OM9LayerTransfer;
public static class MatrixLayerAbiProof {
    public static int Main(string[] args) {
        try {
            byte[] source=File.ReadAllBytes(args[0]),destination=File.ReadAllBytes(args[1]),bindings=File.ReadAllBytes(args[2]);
            byte[] g=Encoding.ASCII.GetBytes("3D Geometry File Format        5\nABI fixture, not a native geometry acceptance");
            NativeLayerApi.VerifyAbi();NativeLayerApi.Validate(source);
            byte[] metadata=NativeLayerApi.Prepare(source,1,g);NativeLayerApi.Capture(metadata,g);
            byte[] plan=NativeLayerApi.Receive(metadata,g,destination,bindings);
            File.WriteAllBytes(args[3],plan);
            var json=new JavaScriptSerializer();var p=json.Deserialize<Dictionary<string,object>>(Encoding.UTF8.GetString(plan));
            var after=(Dictionary<string,object>)p["after"];var layers=(System.Collections.ArrayList)after["layers"];
            if(layers.Count!=33)throw new Exception("Full empty palette lost across real managed/native ABI");
            NativeLayerApi.CheckCurrent(destination,destination);
            Expect(12,delegate {NativeLayerApi.CheckCurrent(destination,source);});
            byte[] changed=(byte[])g.Clone();changed[changed.Length-1]^=1;
            Expect(17,delegate {NativeLayerApi.Capture(metadata,changed);});
            Expect(17,delegate {NativeLayerApi.Receive(metadata,g,destination,Encoding.UTF8.GetBytes("[]"));});
            Console.WriteLine("Actual x64 C# -> Rust ABI PASS: validation, full palette, source-wins, generation, binding, stale and altered-byte rejection. Native Rhino commit unverified.");
            return 0;
        } catch(Exception e) {Console.WriteLine(e);return 1;}
    }
    static void Expect(uint code,Action run) {try {run();}catch(InvalidOperationException e){if(e.Message=="OM9 Rust layer error "+code)return;throw;}throw new Exception("Expected Rust error "+code);}
}
