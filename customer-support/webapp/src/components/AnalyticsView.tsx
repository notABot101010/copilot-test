import { useEffect, useState } from 'preact/hooks';
import { IconGlobe, IconDeviceDesktop, IconFile } from '@tabler/icons-react';
import { analytics, setAnalytics, currentWorkspace } from '../state';
import * as api from '../services/api';
import type { PageStats, CountryStats, BrowserStats } from '../types';

export function AnalyticsView() {
  const workspace = currentWorkspace.value;
  const [days, setDays] = useState(30);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (workspace) {
      loadAnalytics();
    }
  }, [workspace?.id, days]);

  async function loadAnalytics() {
    if (!workspace) return;
    setLoading(true);
    try {
      const data = await api.getAnalytics(workspace.id, days);
      setAnalytics(data);
    } catch (err) {
      console.error('Failed to load analytics:', err);
    } finally {
      setLoading(false);
    }
  }

  if (!workspace) {
    return (
      <div className="flex-1 flex items-center justify-center bg-gray-50">
        <p className="text-gray-500">No workspace selected</p>
      </div>
    );
  }

  const data = analytics.value;

  return (
    <div className="flex-1 p-6 bg-gray-50 overflow-y-auto">
      <div className="max-w-6xl mx-auto">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold text-gray-900">Analytics</h1>
          <select
            value={days}
            onChange={(e) => setDays(Number((e.target as HTMLSelectElement).value))}
            className="px-4 py-2 border border-gray-300 rounded-lg bg-white focus:outline-none focus:border-blue-500"
          >
            <option value={7}>Last 7 days</option>
            <option value={30}>Last 30 days</option>
            <option value={90}>Last 90 days</option>
          </select>
        </div>

        {loading ? (
          <div className="flex items-center justify-center py-12">
            <p className="text-gray-500">Loading analytics...</p>
          </div>
        ) : !data ? (
          <div className="flex items-center justify-center py-12">
            <p className="text-gray-500">No analytics data available</p>
          </div>
        ) : (
          <>
            {/* Summary cards */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
              <div className="bg-white rounded-lg p-6 shadow-sm">
                <h3 className="text-sm font-medium text-gray-500 mb-1">Total Visitors</h3>
                <p className="text-3xl font-bold text-gray-900">{data.total_visitors}</p>
              </div>
              <div className="bg-white rounded-lg p-6 shadow-sm">
                <h3 className="text-sm font-medium text-gray-500 mb-1">Total Page Views</h3>
                <p className="text-3xl font-bold text-gray-900">{data.total_page_views}</p>
              </div>
            </div>

            {/* Analytics tables */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              {/* Top Pages */}
              <div className="bg-white rounded-lg shadow-sm">
                <div className="p-4 border-b border-gray-200 flex items-center gap-2">
                  <IconFile size={20} className="text-gray-500" />
                  <h2 className="font-semibold text-gray-900">Top Pages</h2>
                </div>
                <div className="divide-y divide-gray-100">
                  {data.top_pages.length === 0 ? (
                    <div className="p-4 text-center text-gray-500 text-sm">
                      No page views yet
                    </div>
                  ) : (
                    data.top_pages.map((page: PageStats, index: number) => (
                      <div
                        key={index}
                        className="p-4 flex items-center justify-between hover:bg-gray-50"
                      >
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium text-gray-900 truncate">
                            {page.page_url}
                          </p>
                          <p className="text-xs text-gray-500">
                            {page.page_views} views
                          </p>
                        </div>
                        <span className="ml-2 text-sm font-semibold text-gray-600">
                          {page.visitors}
                        </span>
                      </div>
                    ))
                  )}
                </div>
              </div>

              {/* Top Countries */}
              <div className="bg-white rounded-lg shadow-sm">
                <div className="p-4 border-b border-gray-200 flex items-center gap-2">
                  <IconGlobe size={20} className="text-gray-500" />
                  <h2 className="font-semibold text-gray-900">Top Countries</h2>
                </div>
                <div className="divide-y divide-gray-100">
                  {data.top_countries.length === 0 ? (
                    <div className="p-4 text-center text-gray-500 text-sm">
                      No country data yet
                    </div>
                  ) : (
                    data.top_countries.map((country: CountryStats, index: number) => (
                      <div
                        key={index}
                        className="p-4 flex items-center justify-between hover:bg-gray-50"
                      >
                        <p className="text-sm font-medium text-gray-900">
                          {country.country}
                        </p>
                        <span className="text-sm font-semibold text-gray-600">
                          {country.visitors}
                        </span>
                      </div>
                    ))
                  )}
                </div>
              </div>

              {/* Top Browsers */}
              <div className="bg-white rounded-lg shadow-sm">
                <div className="p-4 border-b border-gray-200 flex items-center gap-2">
                  <IconDeviceDesktop size={20} className="text-gray-500" />
                  <h2 className="font-semibold text-gray-900">Top Browsers</h2>
                </div>
                <div className="divide-y divide-gray-100">
                  {data.top_browsers.length === 0 ? (
                    <div className="p-4 text-center text-gray-500 text-sm">
                      No browser data yet
                    </div>
                  ) : (
                    data.top_browsers.map((browser: BrowserStats, index: number) => (
                      <div
                        key={index}
                        className="p-4 flex items-center justify-between hover:bg-gray-50"
                      >
                        <p className="text-sm font-medium text-gray-900">
                          {browser.browser}
                        </p>
                        <span className="text-sm font-semibold text-gray-600">
                          {browser.visitors}
                        </span>
                      </div>
                    ))
                  )}
                </div>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
