import { Text, UnstyledButton, Collapse } from '@mantine/core';
import { useState } from 'preact/hooks';
import { IconHash, IconVolume, IconChevronDown, IconSettings, IconPlus, IconSpeakerphone } from '@tabler/icons-react';
import type { Channel, Server } from '../types';

interface ChannelListProps {
  server: Server;
  channels: Channel[];
  selectedChannelId: string | null;
  onSelectChannel: (id: string) => void;
}

function groupChannelsByCategory(channels: Channel[]): Map<string, Channel[]> {
  const grouped = new Map<string, Channel[]>();
  
  channels.forEach(channel => {
    const category = channel.category || 'General';
    if (!grouped.has(category)) {
      grouped.set(category, []);
    }
    grouped.get(category)!.push(channel);
  });
  
  return grouped;
}

function ChannelIcon({ type }: { type: Channel['type'] }) {
  switch (type) {
    case 'voice':
      return <IconVolume size={18} className="text-[#80848e] shrink-0" />;
    case 'announcement':
      return <IconSpeakerphone size={18} className="text-[#80848e] shrink-0" />;
    default:
      return <IconHash size={18} className="text-[#80848e] shrink-0" />;
  }
}

export function ChannelList({ server, channels, selectedChannelId, onSelectChannel }: ChannelListProps) {
  const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set());
  const groupedChannels = groupChannelsByCategory(channels);
  
  const toggleCategory = (category: string) => {
    setCollapsedCategories(prev => {
      const next = new Set(prev);
      if (next.has(category)) {
        next.delete(category);
      } else {
        next.add(category);
      }
      return next;
    });
  };

  return (
    <div className="flex flex-col h-full bg-[#2b2d31] w-full">
      {/* Server header */}
      <div className="flex items-center justify-between px-4 h-12 border-b border-[#1f2023] shadow-sm cursor-pointer hover:bg-[#35373c]">
        <Text fw={600} size="sm" className="text-white truncate">
          {server.name}
        </Text>
        <IconChevronDown size={18} className="text-[#b5bac1]" />
      </div>
      
      {/* Channels list */}
      <div className="flex-1 overflow-y-auto pt-4 px-2">
        {Array.from(groupedChannels.entries()).map(([category, categoryChannels]) => (
          <div key={category} className="mb-4">
            {/* Category header */}
            <UnstyledButton
              onClick={() => toggleCategory(category)}
              className="flex items-center gap-1 px-1 mb-1 w-full group"
            >
              <IconChevronDown
                size={12}
                className={`text-[#949ba4] transition-transform ${
                  collapsedCategories.has(category) ? '-rotate-90' : ''
                }`}
              />
              <Text size="xs" fw={600} className="text-[#949ba4] uppercase tracking-wide group-hover:text-[#dbdee1]">
                {category}
              </Text>
              <IconPlus size={14} className="ml-auto text-[#949ba4] opacity-0 group-hover:opacity-100" />
            </UnstyledButton>
            
            {/* Channels in category */}
            <Collapse in={!collapsedCategories.has(category)}>
              {categoryChannels.map(channel => (
                <UnstyledButton
                  key={channel.id}
                  onClick={() => onSelectChannel(channel.id)}
                  className={`flex items-center gap-1.5 px-2 py-1.5 rounded w-full mb-0.5 group ${
                    selectedChannelId === channel.id
                      ? 'bg-[#404249] text-white'
                      : 'text-[#949ba4] hover:bg-[#35373c] hover:text-[#dbdee1]'
                  }`}
                >
                  <ChannelIcon type={channel.type} />
                  <Text size="sm" className="truncate flex-1">
                    {channel.name}
                  </Text>
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100">
                    <IconSettings size={14} className="text-[#b5bac1]" />
                  </div>
                </UnstyledButton>
              ))}
            </Collapse>
          </div>
        ))}
      </div>
      
      {/* User area */}
      <div className="h-[52px] bg-[#232428] px-2 flex items-center gap-2">
        <div className="w-8 h-8 rounded-full bg-[#5865f2]" />
        <div className="flex-1 min-w-0">
          <Text size="xs" fw={600} className="text-white truncate">You</Text>
          <Text size="xs" className="text-[#949ba4] truncate">Online</Text>
        </div>
        <IconSettings size={18} className="text-[#b5bac1] cursor-pointer hover:text-[#dbdee1]" />
      </div>
    </div>
  );
}
