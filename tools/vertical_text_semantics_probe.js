(function installVerticalTextSemanticsProbe(global){
  "use strict";
  const rect=value=>[value.x,value.y,value.width,value.height];
  const ranges=node=>{
    const text=node.firstChild;if(!text)return [];
    const output=[];
    for(let offset=0;offset<text.length;offset++){
      const range=document.createRange();range.setStart(text,offset);range.setEnd(text,offset+1);
      output.push([offset,text.data.slice(offset,offset+1),...rect(range.getBoundingClientRect())]);
    }
    return output;
  };
  const sample=(style,text)=>{
    const node=document.createElement("div");
    node.style.cssText=`font:20px Arial;width:80px;height:100px;margin:0;padding:0;border:0;${style}`;
    node.textContent=text;document.body.append(node);
    const range=document.createRange();range.selectNodeContents(node);
    const computed=getComputedStyle(node);
    const result={computed:[computed.writingMode,computed.textOrientation,computed.textCombineUpright],element:rect(node.getBoundingClientRect()),range:rect(range.getBoundingClientRect()),fragments:Array.from(range.getClientRects(),rect),units:ranges(node)};
    node.remove();return result;
  };
  global.runVerticalTextSemanticsProbe=function(){
    document.body.replaceChildren();document.body.style.cssText="margin:8px";
    const output=Object.create(null);
    output.mixed=sample("writing-mode:vertical-rl;text-orientation:mixed","AB中");
    output.upright=sample("writing-mode:vertical-rl;text-orientation:upright","AB中");
    output.sideways=sample("writing-mode:vertical-rl;text-orientation:sideways","AB中");
    output.punctuation=sample("writing-mode:vertical-rl","「中」、。！（）");
    output.wrapRl=sample("writing-mode:vertical-rl;width:70px;height:45px","ABCDEFGHIJ");
    output.wrapLr=sample("writing-mode:vertical-lr;width:70px;height:45px","ABCDEFGHIJ");
    output.cjkWrap=sample("writing-mode:vertical-rl;width:70px;height:45px","中文天地玄黄");
    output.mixedWrap=sample("writing-mode:vertical-rl;width:70px;height:45px","中A中A中A");
    output.latinMixed=sample("writing-mode:vertical-rl;text-orientation:mixed","ABC");
    output.latinUpright=sample("writing-mode:vertical-rl;text-orientation:upright","ABC");
    output.cjkUpright=sample("writing-mode:vertical-rl;text-orientation:upright","中文");

    const combined=document.createElement("div");combined.style.cssText="font:20px Arial;writing-mode:vertical-rl;width:80px;height:100px";
    const before=document.createTextNode("前"),span=document.createElement("span"),after=document.createTextNode("後");
    span.style.textCombineUpright="all";span.textContent="12";combined.append(before,span,after);document.body.append(combined);
    const combinedRange=document.createRange();combinedRange.selectNodeContents(span);
    output.combine={computed:getComputedStyle(span).textCombineUpright,host:rect(combined.getBoundingClientRect()),span:rect(span.getBoundingClientRect()),range:rect(combinedRange.getBoundingClientRect()),units:ranges(span)};combined.remove();

    const ruby=document.createElement("ruby");ruby.style.cssText="font:20px Arial;writing-mode:vertical-rl;width:80px;height:100px";
    ruby.append(document.createTextNode("漢"));const rt=document.createElement("rt");rt.textContent="かん";ruby.append(rt);document.body.append(ruby);
    const rubyRange=document.createRange();rubyRange.selectNodeContents(ruby);
    output.ruby={display:[getComputedStyle(ruby).display,getComputedStyle(rt).display],ruby:rect(ruby.getBoundingClientRect()),rt:rect(rt.getBoundingClientRect()),range:rect(rubyRange.getBoundingClientRect()),fragments:Array.from(rubyRange.getClientRects(),rect)};ruby.remove();

    const rubySample=(writingMode,position)=>{
      const host=document.createElement("div");host.style.cssText=`font:20px Arial;writing-mode:${writingMode};width:100px;height:100px`;
      const ruby=document.createElement("ruby");ruby.style.rubyPosition=position;ruby.append(document.createTextNode("\u6f22"));
      const rt=document.createElement("rt");rt.textContent="\u304b\u3093";ruby.append(rt);host.append("\u524d",ruby,"\u5f8c");document.body.append(host);
      const range=document.createRange();range.selectNodeContents(host);
      const result={computed:[getComputedStyle(ruby).rubyPosition,getComputedStyle(rt).fontSize],host:rect(host.getBoundingClientRect()),ruby:rect(ruby.getBoundingClientRect()),rt:rect(rt.getBoundingClientRect()),range:rect(range.getBoundingClientRect()),fragments:Array.from(range.getClientRects(),rect)};
      host.remove();return result;
    };
    output.rubyEmbeddedOver=rubySample("vertical-rl","over");
    output.rubyEmbeddedUnder=rubySample("vertical-rl","under");
    output.rubyHorizontal=rubySample("horizontal-tb","over");
    return output;
  };
})(globalThis);
