import React, { useState } from 'react';
import {
  Bell,
  CheckCircle2,
  AlertTriangle,
  Clock,
  Check,
  Plus,
  Sliders,
} from 'lucide-react';
import { NotificationItem, Priority } from '../types';
import { api } from '../services/api';

interface NotificationsPageProps {
  notifications: NotificationItem[];
  onRefresh: () => void;
}

export const NotificationsPage: React.FC<NotificationsPageProps> = ({
  notifications,
  onRefresh,
}) => {
  const [filterUnread, setFilterUnread] = useState(false);

  const displayed = filterUnread
    ? notifications.filter((n) => !n.is_read)
    : notifications;

  const markRead = async (id: string) => {
    try {
      await api.markNotificationRead(id);
      onRefresh();
    } catch (e) {
      console.warn('Failed to mark notification read:', e);
    }
  };

  return (
    <div className="space-y-4">
      {/* Header Bar */}
      <div className="bg-[#181A20] border border-[#2C303A] rounded-xl p-4 flex items-center justify-between">
        <div>
          <h3 className="text-sm font-bold text-[#F2F3F5]">Notifications & Alert Feed</h3>
          <p className="text-xs text-[#A1A6B2]">
            Alerts dispatched according to your configured file change rules and system events.
          </p>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={() => setFilterUnread(!filterUnread)}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold border transition-colors ${
              filterUnread
                ? 'bg-[#6C8CFF] text-white border-[#6C8CFF]'
                : 'bg-[#20232B] text-[#A1A6B2] border-[#2C303A] hover:text-[#F2F3F5]'
            }`}
          >
            {filterUnread ? 'Showing Unread Only' : 'Show All'}
          </button>
        </div>
      </div>

      {/* Notifications List */}
      <div className="space-y-2.5">
        {displayed.length === 0 ? (
          <div className="p-12 bg-[#181A20] border border-[#2C303A] rounded-xl text-center space-y-2">
            <Bell className="w-8 h-8 text-[#A1A6B2] mx-auto opacity-50" />
            <h4 className="text-sm font-bold text-[#F2F3F5]">No Notifications</h4>
            <p className="text-xs text-[#A1A6B2]">
              You're all caught up! Change alerts will appear here as filesystem rules trigger.
            </p>
          </div>
        ) : (
          displayed.map((item) => (
            <div
              key={item.id}
              className={`bg-[#181A20] border rounded-xl p-4 flex items-start justify-between transition-colors ${
                item.is_read
                  ? 'border-[#2C303A] opacity-75'
                  : 'border-[#6C8CFF]/50 bg-[#20232B]/40'
              }`}
            >
              <div className="flex items-start space-x-3">
                <div className="p-2 rounded-lg bg-[#20232B] mt-0.5">
                  <Bell className="w-4 h-4 text-[#6C8CFF]" />
                </div>
                <div className="space-y-1">
                  <div className="flex items-center space-x-2">
                    <span className="text-sm font-bold text-[#F2F3F5]">{item.title}</span>
                    <span
                      className={`text-[9px] font-bold px-1.5 py-0.2 rounded ${
                        item.priority === 'URGENT' || item.priority === 'HIGH'
                          ? 'bg-[#EA6868]/20 text-[#EA6868]'
                          : 'bg-[#6C8CFF]/20 text-[#6C8CFF]'
                      }`}
                    >
                      {item.priority}
                    </span>
                    {!item.is_read && (
                      <span className="w-2 h-2 rounded-full bg-[#6C8CFF] inline-block" />
                    )}
                  </div>
                  <p className="text-xs text-[#A1A6B2]">{item.message}</p>
                  <div className="text-[10px] text-[#A1A6B2] font-mono pt-1">
                    {new Date(item.created_at).toLocaleString()}
                  </div>
                </div>
              </div>

              {!item.is_read && (
                <button
                  onClick={() => markRead(item.id)}
                  title="Mark as read"
                  className="p-1.5 rounded-lg bg-[#20232B] hover:bg-[#6C8CFF] text-[#A1A6B2] hover:text-white border border-[#2C303A] transition-colors"
                >
                  <Check className="w-4 h-4" />
                </button>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
};
