// Experimental, exact-version x86 V8 ABI. Never packaged as a product adapter.
'use strict';
if(Process.arch!=='ia32')throw new Error('Requires verified x86 ABI');
const mod=Process.getModuleByName('nw.dll');
const fn=(name,ret,args,abi)=>new NativeFunction(mod.getExportByName(name),ret,args,abi||'mscdecl');
const version=fn('?GetVersion@V8@v8@@SAPBDXZ','pointer',[])().readUtf8String();
if(version!=='6.5.254.31')throw new Error('Unsupported V8 version: '+version);
const current=fn('?GetCurrent@Isolate@v8@@SAPAV12@XZ','pointer',[]);
const scopeNew=fn('??0HandleScope@v8@@QAE@PAVIsolate@1@@Z','void',['pointer','pointer'],'thiscall');
const scopeDelete=fn('??1HandleScope@v8@@QAE@XZ','void',['pointer'],'thiscall');
const catcherNew=fn('??0TryCatch@v8@@QAE@PAVIsolate@1@@Z','void',['pointer','pointer'],'thiscall');
const catcherDelete=fn('??1TryCatch@v8@@QAE@XZ','void',['pointer'],'thiscall');
const stringNew=fn('?NewFromUtf8@String@v8@@SA?AV?$MaybeLocal@VString@v8@@@2@PAVIsolate@2@PBDW4NewStringType@2@H@Z','pointer',['pointer','pointer','pointer','int','int']);
const compile=fn('?Compile@Script@v8@@SA?AV?$MaybeLocal@VScript@v8@@@2@V?$Local@VContext@v8@@@2@V?$Local@VString@v8@@@2@PAVScriptOrigin@2@@Z','pointer',['pointer','pointer','pointer','pointer']);
const run=fn('?Run@Script@v8@@QAE?AV?$MaybeLocal@VValue@v8@@@2@V?$Local@VContext@v8@@@2@@Z','pointer',['pointer','pointer','pointer'],'thiscall');
const utf8New=fn('??0Utf8Value@String@v8@@QAE@PAVIsolate@2@V?$Local@VValue@v8@@@2@@Z','void',['pointer','pointer','pointer'],'thiscall');
const utf8Delete=fn('??1Utf8Value@String@v8@@QAE@XZ','void',['pointer'],'thiscall');
let request=null,busy=false,hits=0;
const listener=Interceptor.attach(mod.getExportByName('?Call@Function@v8@@QAE?AV?$MaybeLocal@VValue@v8@@@2@V?$Local@VContext@v8@@@2@V?$Local@VValue@v8@@@2@HQAV52@@Z'),{
 onEnter(args){
  hits++;
  if(busy||!request)return;
  const isolate=current(),ctx=args[1];
  if(isolate.isNull()||ctx.isNull())return;
  const task=request;busy=true;
  const scope=Memory.alloc(64),catcher=Memory.alloc(256);
  let scopeLive=false,catchLive=false;
  try{
   scopeNew(scope,isolate);scopeLive=true;catcherNew(catcher,isolate);catchLive=true;
   const source=Memory.allocUtf8String('JSON.stringify((function(){try{if(typeof Utils==="undefined"||Utils.RPGMAKER_NAME!=="MV"||typeof document==="undefined"||document.title!=="Glyphshift MV Contract")return {error:"Wrong fixture context"};'+task.source+'}catch(e){return {error:String(e)}}})())');
   const str=Memory.alloc(Process.pointerSize),script=Memory.alloc(Process.pointerSize),result=Memory.alloc(Process.pointerSize);
   stringNew(str,isolate,source,0,-1);if(str.readPointer().isNull())throw new Error('No string');
   compile(script,ctx,str.readPointer(),ptr(0));if(script.readPointer().isNull())throw new Error('Compile failed');
   run(script.readPointer(),result,ctx);if(result.readPointer().isNull())throw new Error('Run failed');
   const text=Memory.alloc(32);utf8New(text,isolate,result.readPointer());
   try{
    const data=text.readPointer(),value=data.isNull()?null:data.readUtf8String();
    // Node and the page may both enter Function::Call. A different realm must
    // not consume the request intended for the authored game page.
    if(value && JSON.parse(value).error==='Wrong fixture context')return;
    request=null;clearTimeout(task.timer);task.resolve(value);
   }finally{utf8Delete(text);}
  }catch(e){request=null;clearTimeout(task.timer);task.reject(String(e));}
  finally{if(catchLive)catcherDelete(catcher);if(scopeLive)scopeDelete(scope);busy=false;}
 }
});
let runtimeModule=null,runtimePath=null;
let bridgePath=null;
const nodeLoadListener=Interceptor.attach(Process.getModuleByName('node.dll').getExportByName('uv_dlopen'),{
 onEnter(args){this.registerBridge=bridgePath!==null && args[0].readUtf8String().replace(/\//g,'\\').toLowerCase()===bridgePath;},
 onLeave(result){
  if(!this.registerBridge||result.toInt32()!==0)return;
  if(current().isNull())throw new Error('Node registration requires the engine thread');
  const bridge=Process.getModuleByName('glyphshift_mv_fixture.dll');
  const register=new NativeFunction(bridge.getExportByName('glyphshift_mv_register_node_v1'),'int',[],'mscdecl');
  const status=register();if(status!==0)throw new Error('Bridge registration refused: '+status);
 }
});
function runtimeCommand(path,operation,json){
 if(!runtimeModule){
  const load=new NativeFunction(Process.getModuleByName('kernel32.dll').getExportByName('LoadLibraryW'),'pointer',['pointer'],'stdcall');
  if(load(Memory.allocUtf16String(path)).isNull())throw new Error('Runtime load failed');
  runtimeModule=Process.getModuleByName('glyphshift_target_runtime.dll');runtimePath=path;
 }
 if(path!==runtimePath)throw new Error('Runtime path changed within session');
 if(operation==='preload'){
  if(!bridgePath)throw new Error('Fixture deployment must be validated first');
  const load=new NativeFunction(Process.getModuleByName('kernel32.dll').getExportByName('LoadLibraryW'),'pointer',['pointer'],'stdcall');
  return {status:load(Memory.allocUtf16String(bridgePath)).isNull()?1:0};
 }
 if(operation==='activate'){
  const deployment=JSON.parse(json);
  if(deployment.adapters.length!==1||deployment.adapters[0].adapter_id!=='fixture.rpgmaker-mv.runtime-bridge')throw new Error('Expected the authored fixture bridge');
  bridgePath=deployment.adapters[0].library.replace(/\//g,'\\').toLowerCase();
 }
 const symbols={activate:'glyphshift_runtime_activate_v1',update:'glyphshift_runtime_update_v1',capture:'glyphshift_runtime_observation_query_v1',deactivate:'glyphshift_runtime_deactivate_v1'};
 if(!Object.prototype.hasOwnProperty.call(symbols,operation))throw new Error('Unknown Runtime operation');
 const call=new NativeFunction(runtimeModule.getExportByName(symbols[operation]),'uint32',['pointer'],'stdcall');
 if(operation==='deactivate')return {status:call(ptr(0))};
 if(operation==='capture'){
  const capacity=1024*1024,output=Memory.alloc(capacity),query=Memory.alloc(16);
  query.writeU32(16);query.add(4).writePointer(output);query.add(8).writeU32(capacity);query.add(12).writeU32(0);
  const status=call(query),length=query.add(12).readU32();
  return {status,batch:status===0&&length<=capacity?JSON.parse(output.readUtf8String(length)):null};
 }
 if(typeof json!=='string'||json.length>131072)throw new Error('Invalid Runtime command');
 const data=Memory.allocUtf8String(json),command=Memory.alloc(12);
 command.writeU32(12);command.add(4).writePointer(data);command.add(8).writeU32(unescape(encodeURIComponent(json)).length);
 return {status:call(command)};
}
rpc.exports={
 runtime:runtimeCommand,
 evaluate(source){if(typeof source!=='string'||source.length>131072)throw new Error('Invalid source');if(request)throw new Error('Request in flight');return new Promise((resolve,reject)=>{const task={source,resolve,reject};task.timer=setTimeout(()=>{if(request===task){request=null;reject('Engine callback timeout');}},5000);request=task;});},
 status(){return {hits,pending:!!request};},
 stop(){listener.detach();nodeLoadListener.detach();if(request){clearTimeout(request.timer);request.reject('Stopped');request=null;}}
};
