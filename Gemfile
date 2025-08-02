# Gemfile for Caxton Jekyll Site
# GitHub Pages compatible configuration

source "https://rubygems.org"

# Use GitHub Pages gem for consistency
gem "github-pages", group: :jekyll_plugins

# Additional plugins for enhanced functionality
group :jekyll_plugins do
  gem "jekyll-feed"
  gem "jekyll-sitemap"
  gem "jekyll-seo-tag"
  gem "jekyll-paginate-v2"
  gem "jekyll-relative-links"
  gem "jekyll-optional-front-matter"
  gem "jekyll-readme-index"
  gem "jekyll-titles-from-headings"
end

# Development dependencies
group :development do
  gem "webrick"
end

# Platform-specific gems
platforms :mingw, :x64_mingw, :mswin, :jruby do
  gem "tzinfo", ">= 1", "< 3"
  gem "tzinfo-data"
end

# Lock `http_parser.rb` gem to `v0.6.x` on JRuby builds since newer versions
gem "http_parser.rb", "~> 0.6.0", :platforms => [:jruby]