// Boundary model matches the installed Rhino 5 IL: SetUserString writes a
// ConstPointer; CommitChanges rejects a still document-controlled layer.
// Execute the real native helper against this model; native Matrix proof is separate.
using System;
namespace Rhino.DocObjects {
 public sealed class Layer : IDisposable {
  sealed class Data {public string Witness;}
  Data data=new Data();
  public Guid Id; public string Witness {get{return data.Witness;}set{data.Witness=value;}} public bool IsDocumentControlled=true;
  public Layer Wrap(){return new Layer {Id=Id,data=data};}
  public void EnsurePrivateCopy(){data=new Data {Witness=data.Witness};IsDocumentControlled=false;}
  public string GetUserString(string key){return Witness;}
  public bool SetUserString(string key,string value){Witness=value;return true;}
  public bool CommitChanges(){return !IsDocumentControlled;}
  public void Dispose(){}
 }
}
namespace Rhino {
 public sealed class LayerTable {
  public DocObjects.Layer Live=new DocObjects.Layer {Id=Guid.NewGuid(),Witness="new"};
  public string UndoWitness="new";public int Writes;public bool Reject;
  public int Find(Guid id,bool quiet){return id==Live.Id?0:-1;}
  public DocObjects.Layer this[int i]{get {if(i!=0)throw new Exception("Invalid index");return Live.Wrap();}}
  public bool Modify(DocObjects.Layer settings,int i,bool quiet){
   if(Reject)return false;UndoWitness=Live.Witness;Live.Witness=settings.Witness;Writes++;return true;
  }
 }
 public sealed class RhinoDoc {public LayerTable Layers=new LayerTable();}
}
class MatrixWitnessContract {
 static void Require(bool v,string name){if(!v)throw new Exception(name);}
 static void Main(){
  var doc=new Rhino.RhinoDoc();var layer=doc.Layers.Live;
  var id=layer.Id;
  OM9LayerTransfer.NativeLayerWitness.Restore(doc,id,null);
  Require(layer.Witness==null,"Absent witness restored");
  Require(doc.Layers.UndoWitness=="new","Native table sees old value before mutation for Undo");
  Require(doc.Layers.Writes==1,"One table commit");
  OM9LayerTransfer.NativeLayerWitness.Restore(doc,id,null);
  Require(doc.Layers.Writes==1,"Unchanged absent witness has no write");
  OM9LayerTransfer.NativeLayerWitness.Restore(doc,id,"");
  Require(layer.Witness==""&&doc.Layers.Writes==2,"Empty witness distinct from absence");
  doc.Layers.Reject=true;bool failed=false;
  try{OM9LayerTransfer.NativeLayerWitness.Restore(doc,id,"rejected");}catch(InvalidOperationException){failed=true;}
  Require(failed&&layer.Witness=="","Failed commit leaves live witness intact");
  failed=false;try{OM9LayerTransfer.NativeLayerWitness.Restore(doc,Guid.NewGuid(),"x");}catch(InvalidOperationException){failed=true;}
  Require(failed&&layer.Witness=="","Missing layer refuses before write");
  Console.WriteLine("Native witness contract: PASS; five cases; installed SDK native application still pending.");
 }
}
