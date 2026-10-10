#include "LayerRustAbi.h"
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

static int checks=0;
static void check(bool value,const char* name){++checks;if(!value)throw std::runtime_error(name);}
static Om9LayerByteView view(const std::string& s){return {reinterpret_cast<const unsigned char*>(s.data()),s.size()};}
static std::string text(std::uint64_t snapshot,std::uint32_t kind,std::size_t index,std::uint32_t field){
    std::size_t length=0;auto code=om9_layer_snapshot_text(snapshot,kind,index,field,nullptr,0,&length);
    check(code==0||code==15,"text size status");std::string value(length,'\0');
    check(om9_layer_snapshot_text(snapshot,kind,index,field,reinterpret_cast<unsigned char*>(value.data()),value.size(),&length)==0,"text copy");return value;
}
int main(){
    try{
        std::string doc="source",empty="",metal="m",gem="g",metalName="Metal 01",gemName="Empty gem",id="ring";
        Om9LayerView layers[]={
            {view(metal),view(empty),view(metalName),view(metalName),{34,153,135},1,1,-1,-1,0},
            {view(gem),view(empty),view(gemName),view(gemName),{12,23,244},0,1,-1,-1,0}};
        Om9LayerObjectView objects[]={{view(id),view(metal),{8,9,10},0,1,1,{0,0}}};
        Om9LayerSnapshotView source{1,0,view(doc),9,view(metal),layers,2,objects,1};
        std::uint64_t s=0;check(om9_layer_snapshot_create(&source,&s)==0&&s!=0,"native create snapshot");
        // Rust copied data. Changing caller metadata must not alter the record.
        layers[0].locked=0;layers[0].rgb[0]=255;
        Om9LayerEffective effective{};
        check(om9_layer_snapshot_effective(s,view(id),&effective)==0,"native effective query");
        check(effective.locked==1&&effective.selectable==0&&effective.snap_eligible==1,"separate inherited lock");
        check(effective.rgb[0]==34,"copied layer color");
        // Header fixture tests transport policy only, not native SDK parsing.
        const std::string geometry="3D Geometry File Format       50layer-ABI-fixture";
        std::uint64_t binding=0,received=0;
        check(om9_layer_clipboard_prepare(s,1,view(geometry),&binding)==0,"native binding prepare");
        check(om9_layer_snapshot_free(binding)==13&&om9_layer_plan_free(binding)==13,"native binding type safety");
        std::size_t metadataLength=0;
        check(om9_layer_clipboard_bytes(binding,nullptr,0,&metadataLength)==15,"native binding query");
        std::string metadata(metadataLength,'\0');
        check(om9_layer_clipboard_bytes(binding,reinterpret_cast<unsigned char*>(metadata.data()),metadata.size(),&metadataLength)==0,"native binding copy");
        Om9LayerClipboardInfo clipboardInfo{};
        check(om9_layer_clipboard_receive(view(geometry),view(metadata),1,&received,&clipboardInfo)==0,"native extended receive");
        check(clipboardInfo.version==1&&clipboardInfo.evidence==2&&clipboardInfo.scope==1&&clipboardInfo.geometry_version==50,"native clipboard layout fields");
        check(clipboardInfo.geometry_length==geometry.size()&&clipboardInfo.metadata_length==metadata.size(),"native clipboard layout lengths");
        Om9LayerObjectInfo ownObjectClipboard{};
        check(om9_layer_snapshot_object(received,0,&ownObjectClipboard)==0,"native received object query");
        check(ownObjectClipboard.locked==0,"native transport keeps own lock");
        check(om9_layer_snapshot_free(received)==0,"native receive dispose");
        auto wrongGeometry=geometry;wrongGeometry.back()='x';received=999;
        check(om9_layer_clipboard_receive(view(wrongGeometry),view(metadata),1,&received,&clipboardInfo)==17&&received==0&&clipboardInfo.evidence==0,"native digest rejects before snapshot");
        check(om9_layer_clipboard_receive(view(geometry),view(empty),0,&received,&clipboardInfo)==0&&received==0&&clipboardInfo.evidence==1,"native reduced evidence");
        check(om9_layer_clipboard_free(binding)==0,"native binding dispose");
        std::string targetDoc="destination";Om9LayerSnapshotView destination{1,0,view(targetDoc),0,view(empty),nullptr,0,nullptr,0};
        std::uint64_t d=0,p=0,a=0;
        check(om9_layer_snapshot_create(&destination,&d)==0,"empty native destination");
        check(om9_layer_plan_receive(s,d,2,nullptr,0,&p)==0,"native session receive plan");
        check(om9_layer_plan_validate(p,d)==0,"native current plan");
        check(om9_layer_plan_after_snapshot(p,&a)==0,"native after snapshot");
        Om9LayerCounts counts{};check(om9_layer_snapshot_counts(a,&counts)==0,"native counts");
        check(counts.layer_count==2&&counts.object_count==1&&counts.generation==1,"empty color slot survives session");
        check(text(a,0,0,1)=="g","native active fallback does not unlock metal");
        Om9LayerInfo ownLayer{};check(om9_layer_snapshot_layer(a,0,&ownLayer)==0&&ownLayer.locked==1,"layer lock remains local");
        Om9LayerObjectInfo ownObject{};check(om9_layer_snapshot_object(a,0,&ownObject)==0&&ownObject.locked==0,"object own lock remains separate");
        auto added=text(a,2,0,0);auto addedView=view(added);
        check(om9_layer_snapshot_can_mutate(a,&addedView,1,1,view(empty))==9,"native edit guard");
        check(om9_layer_snapshot_can_mutate(a,&addedView,1,5,view(empty))==0,"native export locked geometry");
        std::size_t required=0;unsigned char mapped[128]{};
        check(om9_layer_plan_map(p,2,view(id),mapped,sizeof(mapped),&required)==0,"native object mapping");
        check(std::string(reinterpret_cast<char*>(mapped),required)==added,"object mapping matches after record");
        destination.generation=1;std::uint64_t stale=0;check(om9_layer_snapshot_create(&destination,&stale)==0,"stale native snapshot");
        check(om9_layer_plan_validate(p,stale)==12,"native stale guard");
        check(om9_layer_snapshot_free(p)==13,"wrong handle type does not release plan");
        check(om9_layer_plan_free(p)==0,"native plan dispose");
        for(auto handle:{s,d,a,stale})check(om9_layer_snapshot_free(handle)==0,"native snapshot dispose");
        check(om9_layer_snapshot_counts(a,&counts)==13,"freed native snapshot rejects");
        std::cout<<"{\"ok\":true,\"checks\":"<<checks<<",\"scope\":\"native C++ to Rust layer ABI\"}\n";return 0;
    }catch(const std::exception& error){std::cerr<<error.what()<<"\n";return 1;}
}
