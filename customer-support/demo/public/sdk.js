"use strict";var CustomerSupport=(()=>{var r=Object.defineProperty;var p=Object.getOwnPropertyDescriptor;var l=Object.getOwnPropertyNames;var h=Object.prototype.hasOwnProperty;var g=(o,e)=>{for(var t in e)r(o,t,{get:e[t],enumerable:!0})},u=(o,e,t,i)=>{if(e&&typeof e=="object"||typeof e=="function")for(let s of l(e))!h.call(o,s)&&s!==t&&r(o,s,{get:()=>e[s],enumerable:!(i=p(e,s))||i.enumerable});return o};var m=o=>u(r({},"__esModule",{value:!0}),o);var v={};g(v,{CustomerSupportWidget:()=>n,default:()=>w});var n=class{constructor(e){this.container=null;this.chatWindow=null;this.isOpen=!1;this.contactId=null;this.conversationId=null;this.messages=[];this.ws=null;this.config={apiUrl:"",position:"bottom-right",primaryColor:"#2563eb",welcomeMessage:"Hi! How can we help you today?",...e},this.visitorId=this.getVisitorId(),document.readyState==="loading"?document.addEventListener("DOMContentLoaded",()=>this.init()):this.init()}getVisitorId(){let e="cs_visitor_id",t=document.cookie.split(";");for(let a of t){let[c,d]=a.trim().split("=");if(c===e)return d}let i="v_"+Math.random().toString(36).substring(2)+Date.now().toString(36),s=new Date(Date.now()+365*24*60*60*1e3);return document.cookie=`${e}=${i}; expires=${s.toUTCString()}; path=/; SameSite=Lax`,i}async init(){this.createStyles(),this.createWidget(),await this.initVisitor(),this.connectWebSocket(),this.trackPageView()}createStyles(){let e=document.createElement("style");e.textContent=`
      .cs-widget-container {
        position: fixed;
        ${this.config.position==="bottom-right"?"right: 20px;":"left: 20px;"}
        bottom: 20px;
        z-index: 999999;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
      }

      .cs-widget-button {
        width: 60px;
        height: 60px;
        border-radius: 50%;
        background-color: ${this.config.primaryColor};
        border: none;
        cursor: pointer;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.2s, box-shadow 0.2s;
      }

      .cs-widget-button:hover {
        transform: scale(1.05);
        box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
      }

      .cs-widget-button svg {
        width: 28px;
        height: 28px;
        fill: white;
      }

      .cs-chat-window {
        position: absolute;
        ${this.config.position==="bottom-right"?"right: 0;":"left: 0;"}
        bottom: 80px;
        width: 380px;
        height: 520px;
        background: white;
        border-radius: 16px;
        box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
        display: none;
        flex-direction: column;
        overflow: hidden;
      }

      .cs-chat-window.open {
        display: flex;
      }

      .cs-chat-header {
        background-color: ${this.config.primaryColor};
        color: white;
        padding: 20px;
        display: flex;
        justify-content: space-between;
        align-items: center;
      }

      .cs-chat-header h3 {
        margin: 0;
        font-size: 18px;
        font-weight: 600;
      }

      .cs-chat-close {
        background: none;
        border: none;
        color: white;
        cursor: pointer;
        padding: 4px;
        opacity: 0.8;
        transition: opacity 0.2s;
      }

      .cs-chat-close:hover {
        opacity: 1;
      }

      .cs-chat-messages {
        flex: 1;
        overflow-y: auto;
        padding: 16px;
        background: #f9fafb;
      }

      .cs-message {
        max-width: 80%;
        margin-bottom: 12px;
        padding: 12px 16px;
        border-radius: 16px;
        font-size: 14px;
        line-height: 1.4;
      }

      .cs-message.visitor {
        margin-left: auto;
        background-color: ${this.config.primaryColor};
        color: white;
        border-bottom-right-radius: 4px;
      }

      .cs-message.agent {
        background-color: white;
        color: #1f2937;
        border-bottom-left-radius: 4px;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
      }

      .cs-message-time {
        font-size: 11px;
        opacity: 0.7;
        margin-top: 4px;
      }

      .cs-chat-input {
        padding: 16px;
        background: white;
        border-top: 1px solid #e5e7eb;
        display: flex;
        gap: 12px;
      }

      .cs-chat-input input {
        flex: 1;
        padding: 12px 16px;
        border: 1px solid #e5e7eb;
        border-radius: 24px;
        font-size: 14px;
        outline: none;
        transition: border-color 0.2s;
      }

      .cs-chat-input input:focus {
        border-color: ${this.config.primaryColor};
      }

      .cs-chat-input button {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background-color: ${this.config.primaryColor};
        border: none;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background-color 0.2s;
      }

      .cs-chat-input button:hover {
        background-color: ${this.config.primaryColor}dd;
      }

      .cs-chat-input button:disabled {
        background-color: #d1d5db;
        cursor: not-allowed;
      }

      .cs-chat-input button svg {
        width: 18px;
        height: 18px;
        fill: white;
      }

      .cs-welcome-message {
        text-align: center;
        padding: 40px 20px;
        color: #6b7280;
      }

      .cs-welcome-message p {
        margin: 0;
        font-size: 15px;
      }

      @media (max-width: 420px) {
        .cs-chat-window {
          width: 100vw;
          height: 100vh;
          bottom: 0;
          right: 0;
          left: 0;
          border-radius: 0;
        }
      }
    `,document.head.appendChild(e)}createWidget(){this.container=document.createElement("div"),this.container.className="cs-widget-container",this.chatWindow=document.createElement("div"),this.chatWindow.className="cs-chat-window",this.chatWindow.innerHTML=`
      <div class="cs-chat-header">
        <h3>Support</h3>
        <button class="cs-chat-close">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>
      <div class="cs-chat-messages">
        <div class="cs-welcome-message">
          <p>${this.config.welcomeMessage}</p>
        </div>
      </div>
      <div class="cs-chat-input">
        <input type="text" placeholder="Type a message..." />
        <button disabled>
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
          </svg>
        </button>
      </div>
    `;let e=document.createElement("button");e.className="cs-widget-button",e.innerHTML=`
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H6l-2 2V4h16v12z"/>
      </svg>
    `,e.addEventListener("click",()=>this.toggle()),this.chatWindow.querySelector(".cs-chat-close")?.addEventListener("click",()=>this.close());let t=this.chatWindow.querySelector("input"),i=this.chatWindow.querySelector(".cs-chat-input button");t.addEventListener("input",()=>{i.disabled=!t.value.trim()}),t.addEventListener("keypress",s=>{s.key==="Enter"&&t.value.trim()&&(this.sendMessage(t.value.trim()),t.value="",i.disabled=!0)}),i.addEventListener("click",()=>{t.value.trim()&&(this.sendMessage(t.value.trim()),t.value="",i.disabled=!0)}),this.container.appendChild(this.chatWindow),this.container.appendChild(e),document.body.appendChild(this.container)}async initVisitor(){try{let e=await fetch(`${this.config.apiUrl}/api/workspaces/${this.config.workspaceId}/visitor/init`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({visitor_id:this.visitorId})});if(!e.ok){console.error("Failed to initialize visitor");return}let t=await e.json();this.contactId=t.contact_id,this.conversationId=t.conversation_id,this.messages=t.messages,this.renderMessages()}catch(e){console.error("Failed to initialize visitor:",e)}}connectWebSocket(){let e=window.location.protocol==="https:"?"wss:":"ws:",t=this.config.apiUrl?new URL(this.config.apiUrl).host:window.location.host;this.ws=new WebSocket(`${e}//${t}/ws/workspaces/${this.config.workspaceId}`),this.ws.onmessage=i=>{try{let s=JSON.parse(i.data);s.type==="new_message"&&s.conversation_id===this.conversationId&&(this.messages.push(s.message),this.renderMessages())}catch(s){console.error("Failed to parse WebSocket message:",s)}},this.ws.onclose=()=>{setTimeout(()=>this.connectWebSocket(),5e3)}}renderMessages(){let e=this.chatWindow?.querySelector(".cs-chat-messages");if(e){if(this.messages.length===0){e.innerHTML=`
        <div class="cs-welcome-message">
          <p>${this.config.welcomeMessage}</p>
        </div>
      `;return}e.innerHTML=this.messages.map(t=>{let i=new Date(t.created_at).toLocaleTimeString([],{hour:"2-digit",minute:"2-digit"});return`
          <div class="cs-message ${t.sender_type}">
            ${t.content}
            <div class="cs-message-time">${i}</div>
          </div>
        `}).join(""),e.scrollTop=e.scrollHeight}}async sendMessage(e){try{let t=await fetch(`${this.config.apiUrl}/api/workspaces/${this.config.workspaceId}/visitor/message`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({visitor_id:this.visitorId,content:e,conversation_id:this.conversationId})});if(!t.ok){console.error("Failed to send message");return}let i=await t.json();this.conversationId=i.conversation_id,this.messages.push(i.message),this.renderMessages()}catch(t){console.error("Failed to send message:",t)}}toggle(){this.isOpen?this.close():this.open()}open(){this.isOpen=!0,this.chatWindow?.classList.add("open"),this.chatWindow?.querySelector("input")?.focus()}close(){this.isOpen=!1,this.chatWindow?.classList.remove("open")}async trackPageView(){try{await fetch(`${this.config.apiUrl}/api/workspaces/${this.config.workspaceId}/track`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({page_url:window.location.pathname,page_title:document.title,referrer:document.referrer||null})})}catch(e){console.error("Failed to track page view:",e)}}};window.CustomerSupport=n;var w=n;return m(v);})();
