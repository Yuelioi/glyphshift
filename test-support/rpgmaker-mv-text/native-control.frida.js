// Test-only DLL loader/control transport. No Interceptor, V8 ABI or game-thread
// execution lives here; those are owned by native-session.cpp.
'use strict';
let bridge=null,runtime=null;
function library(path){
 const load=new NativeFunction(Process.getModuleByName('kernel32.dll').getExportByName('LoadLibraryW'),'pointer',['pointer'],'stdcall');
 const handle=load(Memory.allocUtf16String(path));
 if(handle.isNull())throw new Error('Test library load failed');
 return Process.getModuleByAddress(handle);
}
function native(module,name,ret,args,abi){return new NativeFunction(module.getExportByName(name),ret,args,abi);}
function bytes(text){return unescape(encodeURIComponent(text)).length;}
rpc.exports={
 configure(path){
  if(Process.arch!=='ia32')throw new Error('Verified x86 fixture only');
  bridge=library(path);
  const status=native(bridge,'glyphshift_mv_session_start_v1','int',[],'mscdecl')();
  if(status)throw new Error('Native session start failed: '+status);
 },
 evaluate(source){
  if(!bridge||typeof source!=='string'||bytes(source)>131072)throw new Error('Invalid native session request');
  const output=Memory.alloc(4*1024*1024),written=Memory.alloc(4);
  const call=native(bridge,'glyphshift_mv_session_evaluate_v1','int',['pointer','uint32','pointer','uint32','pointer'],'mscdecl');
  const status=call(Memory.allocUtf8String(source),bytes(source),output,4*1024*1024,written);
  if(status)throw new Error('Native evaluation failed: '+status);
  return output.readUtf8String(written.readU32());
 },
 runtime(path,operation,json){
  if(operation==='session-start'||operation==='session-stop'){
   const name=operation==='session-start'?'glyphshift_mv_session_start_v1':'glyphshift_mv_session_stop_v1';
   return {status:native(bridge,name,'int',[],'mscdecl')()};
  }
  if(!runtime)runtime=library(path);
  if(operation==='preload')return {status:0}; // configure already retains the bridge DLL.
  const symbols={activate:'glyphshift_runtime_activate_v1',update:'glyphshift_runtime_update_v1',capture:'glyphshift_runtime_observation_query_v1',deactivate:'glyphshift_runtime_deactivate_v1'};
  if(!Object.prototype.hasOwnProperty.call(symbols,operation))throw new Error('Unknown Runtime operation');
  const call=native(runtime,symbols[operation],'uint32',['pointer'],'stdcall');
  if(operation==='deactivate')return {status:call(ptr(0))};
  if(operation==='capture'){
   const capacity=4*1024*1024,output=Memory.alloc(capacity),query=Memory.alloc(16);
   query.writeU32(16);query.add(4).writePointer(output);query.add(8).writeU32(capacity);query.add(12).writeU32(0);
   const status=call(query),length=query.add(12).readU32();
   return {status,batch:status===0&&length<=capacity?JSON.parse(output.readUtf8String(length)):null};
  }
  if(typeof json!=='string'||bytes(json)>131072)throw new Error('Invalid Runtime command');
  const data=Memory.allocUtf8String(json),command=Memory.alloc(12);
  command.writeU32(12);command.add(4).writePointer(data);command.add(8).writeU32(bytes(json));
  return {status:call(command)};
 },
 stop(){if(bridge){const status=native(bridge,'glyphshift_mv_session_stop_v1','int',[],'mscdecl')();if(status)throw new Error('Native session stop failed: '+status);}}
};
