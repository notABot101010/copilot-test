import type { Message } from '../types';

interface Props {
  message: Message;
}

export function MessageBubble({ message }: Props) {
  const isUser = message.role === 'user';
  
  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div
        className={`max-w-[80%] px-4 py-3 rounded-lg ${
          isUser
            ? 'bg-blue-600 text-white'
            : 'bg-white border border-gray-200 text-gray-900'
        }`}
      >
        <div className="flex items-center gap-2 mb-1">
          <span className="text-xs opacity-75">
            {isUser ? 'You' : 'Agent'}
          </span>
          <span className="text-xs opacity-50">
            {new Date(message.created_at).toLocaleTimeString()}
          </span>
        </div>
        <div className="whitespace-pre-wrap text-sm">{message.content}</div>
      </div>
    </div>
  );
}
